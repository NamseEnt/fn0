mod identity;
mod segment;
mod spool;
#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bytes::Bytes;
use parking_lot::Mutex;
use tokio::sync::Notify;

pub use identity::SenderId;
use segment::SegmentWriter;
pub use spool::{Gone, Spool, SpoolReport};

use crate::memprof::{self, Arena};
use crate::signal::Signal;

/// Segments are numbered from one so that zero can mean "signy has none of
/// them" in the cursor and in signy's own memory.
const FIRST_SEGMENT: u64 = 1;

#[derive(Clone, Copy, Debug)]
pub struct QueueLimits {
    pub max_bytes: u64,
    pub max_segment_bytes: u64,
    /// How long an open segment may keep collecting before it is closed and
    /// becomes sendable.
    ///
    /// A segment is what a request carries and what an `fsync` covers, so
    /// nothing leaves this machine and nothing is on the device until one
    /// closes. On a busy host the size closes it first; on a quiet one this
    /// does, and it is the floor on how long a record waits.
    pub max_segment_age: Duration,
}

impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024 * 1024,
            max_segment_bytes: 8 * 1024 * 1024,
            max_segment_age: Duration::from_secs(1),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueueStats {
    /// Everything on disk, which is also everything still owed: a segment
    /// signy has answered for is unlinked, so nothing here has reached it.
    pub queued_bytes: u64,
    pub segments: usize,
    pub appended_records: u64,
    /// What applications handed over, before compression. Against the bytes
    /// the sender ships this is the ratio a segment achieved.
    pub appended_bytes: u64,
    pub dropped_bytes: u64,
    pub dropped_segments: u64,
}

/// The disk queue: one stream of segments per signal, under one identity.
///
/// A segment holds one signal's records and nothing else, so the exports
/// compressed together are the ones that look alike, and a request signy takes
/// goes down one ingest path. What that costs is the ordering: numbering runs
/// per signal, so nothing on disk puts two signals' segments in one order and
/// the queue keeps that order in memory instead.
pub struct Queue {
    /// Indexed by `Signal::index`.
    dirs: [PathBuf; Signal::ALL.len()],
    sender: SenderId,
    limits: QueueLimits,
    inner: Mutex<Inner>,
    appended: Notify,
}

struct Inner {
    /// Indexed by `Signal::index`.
    signals: [SignalQueue; Signal::ALL.len()],
    /// The last arrival stamp handed out. Never written down: it orders the
    /// segments this process is holding, and a restart works the order out
    /// again from what the files say.
    stamped: u64,
    stats: QueueStats,
}

/// One signal's segments. Its own numbering, its own open segment, its own age.
struct SignalQueue {
    segments: VecDeque<Held>,
    active: SegmentWriter,
    /// When the open segment took its first record. `None` while it is empty,
    /// so an idle collector does not roll empty segments forever, and the only
    /// honest answer to "is it empty" — the file can still be nothing while
    /// the encoder holds a block's worth.
    active_since: Option<Instant>,
}

struct Held {
    seq: u64,
    bytes: u64,
    /// Where this segment falls among all three signals', oldest first.
    ///
    /// A number is only comparable inside its own signal now, and both the
    /// order segments are sent in and the one they are dropped in are meant to
    /// be the order they stopped collecting. So the queue stamps a segment as
    /// it closes it, and at open reads that order back off the files' own
    /// timestamps, which record the same moment.
    ///
    /// Zero while the segment is open. Only a closed one is ever compared.
    stamp: u64,
}

/// A closed segment, ready to be shipped whole.
///
/// `body` is the file, byte for byte. One zstd stream over every record the
/// segment took, which is exactly what the wire wants, so nothing is parsed,
/// unwrapped or copied on the way out. The signal travels beside it rather
/// than inside it: every record in there is one, and the request says which.
pub struct SealedSegment {
    pub signal: Signal,
    pub seq: u64,
    pub body: Bytes,
}

impl Queue {
    pub fn open(dir: &Path, limits: QueueLimits, level: i32) -> io::Result<Queue> {
        let _tag = memprof::enter(Arena::Queue);
        std::fs::create_dir_all(dir)?;
        segment::sweep_temporaries(dir)?;

        // An identity file that cannot be read is replaced rather than
        // repaired. Keeping the name while the segment numbering restarted
        // would have signy skip every segment under the high-water mark it
        // still held for that name.
        let sender = match identity::load(dir)? {
            Some(sender) => sender,
            None => {
                let sender = SenderId::generate()?;
                identity::store(dir, sender)?;
                sender
            }
        };

        let dirs = Signal::ALL.map(|signal| dir.join(signal.as_str()));
        let mut recovered = Vec::new();
        let mut queued_bytes = 0;
        // From the highest number the signal's directory ever held, not from
        // the highest still there: signy holds a high-water mark per sender
        // and signal, and numbering that went backwards would have it skip
        // every segment under that mark as one it already stored. Taken before
        // recovery runs, so a segment it finds empty and unlinks does not hand
        // its number to the one that follows.
        let mut nexts = [FIRST_SEGMENT; Signal::ALL.len()];
        for (slot, dir) in dirs.iter().enumerate() {
            std::fs::create_dir_all(dir)?;
            segment::sweep_temporaries(dir)?;
            let mut files = segment::list(dir)?;
            if let Some(last) = files.last() {
                nexts[slot] = last.seq + 1;
            }

            // A stream cannot be resumed: the encoder's state went with the
            // process that held it. So the segment a previous run left open is
            // closed where it stopped, and this run starts a fresh one. Only
            // the last segment of a signal can be unfinished — a roll closes
            // and syncs the old segment before it creates the next one.
            if let Some(mut last) = files.pop() {
                match segment::reseal(dir, last.seq, level)? {
                    Some(bytes) => {
                        last.bytes = bytes;
                        files.push(last);
                    }
                    None => tracing::warn!(
                        signal = Signal::ALL[slot].as_str(),
                        segment = last.seq,
                        "a segment a crash left with nothing readable in it is dropped"
                    ),
                }
            }

            queued_bytes += files.iter().map(|file| file.bytes).sum::<u64>();
            recovered.extend(files.into_iter().map(|file| (slot, file)));
        }

        // The order the three signals closed their segments in, back off the
        // one thing that still records it. Ties fall to the signal's own
        // numbering, which is an order even when a filesystem's timestamps are
        // too coarse to be one.
        recovered.sort_by_key(|(slot, file)| (file.modified, *slot, file.seq));

        let mut stamped = 0;
        let mut segments = Signal::ALL.map(|_| VecDeque::new());
        for (slot, file) in recovered {
            stamped += 1;
            segments[slot].push_back(Held {
                seq: file.seq,
                bytes: file.bytes,
                stamp: stamped,
            });
        }

        let mut signals = Vec::with_capacity(Signal::ALL.len());
        for (slot, dir) in dirs.iter().enumerate() {
            let next = nexts[slot];
            let active = SegmentWriter::create(dir, next, level)?;
            segments[slot].push_back(Held {
                seq: next,
                bytes: 0,
                stamp: 0,
            });
            signals.push(SignalQueue {
                segments: std::mem::take(&mut segments[slot]),
                active,
                active_since: None,
            });
        }

        // How far signy has got is not read back from anywhere, because the
        // files say it: one it has answered for was unlinked on the spot. A
        // crash between the answer and the unlink leaves a segment behind, and
        // offering it again costs one request that signy answers unread.
        Ok(Queue {
            dirs,
            sender,
            limits,
            inner: Mutex::new(Inner {
                signals: signals
                    .try_into()
                    .unwrap_or_else(|_| unreachable!("one queue per signal")),
                stamped,
                stats: QueueStats {
                    queued_bytes,
                    ..QueueStats::default()
                },
            }),
            appended: Notify::new(),
        })
    }

    pub fn sender_id(&self) -> SenderId {
        self.sender
    }

    /// Append one OTLP export to its signal's open segment.
    ///
    /// The four byte length goes in front of it here, as writes into the
    /// encoder, rather than by building a buffer that is the header followed
    /// by a copy of the export. The copy was a second allocation the size of
    /// every export that arrives, alive at the same time as the first.
    pub fn append(&self, signal: Signal, payload: &[u8]) -> io::Result<()> {
        self.write_record(signal, payload.len(), |segment| segment.write_all(payload))
    }

    /// The same record, arriving as the pieces hyper read it in.
    ///
    /// A body is a run of frames and the encoder does not care where one ends,
    /// so there is nothing for a contiguous buffer to be for. Making one meant
    /// allocating the whole export a second time and copying every byte of it
    /// into the new place, per request, and the pieces are written here in
    /// order instead.
    pub fn append_chunks(&self, signal: Signal, chunks: &[Bytes]) -> io::Result<()> {
        let bytes = chunks.iter().map(|chunk| chunk.len()).sum();
        self.write_record(signal, bytes, |segment| {
            for chunk in chunks {
                segment.write_all(chunk)?;
            }
            Ok(())
        })
    }

    fn write_record(
        &self,
        signal: Signal,
        bytes: usize,
        write: impl FnOnce(&mut SegmentWriter) -> io::Result<()>,
    ) -> io::Result<()> {
        let _tag = memprof::enter(Arena::Queue);
        let plain_len = (crate::wire::RECORD_HEADER_BYTES + bytes) as u64;
        if plain_len > self.limits.max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "a {plain_len} byte record cannot fit a {} byte queue",
                    self.limits.max_bytes
                ),
            ));
        }

        let mut inner = self.inner.lock();

        // Against the plain length, which is all that is known before the
        // record is compressed. It only ever overstates what the record will
        // occupy, so the queue errs towards keeping room rather than filling.
        // The budget is the whole queue's, not a share per signal: a host that
        // only ships logs should be free to spend all of it on them.
        while inner.stats.queued_bytes + plain_len > self.limits.max_bytes {
            self.drop_oldest(&mut inner)?;
        }

        // After crossing rather than before: the compressed size of a record
        // is not known until it has been written, and the encoder emits in
        // blocks, so a segment overshoots by at most the last block.
        if inner.signals[signal.index()].active.written() >= self.limits.max_segment_bytes {
            self.roll(&mut inner, signal)?;
        }

        let queue = &mut inner.signals[signal.index()];
        queue.active.write_all(&(bytes as u32).to_le_bytes())?;
        write(&mut queue.active)?;
        if queue.active_since.is_none() {
            queue.active_since = Some(Instant::now());
        }

        let written = queue.active.written();
        let back = queue
            .segments
            .back_mut()
            .expect("a signal always holds its active segment");
        let grew = written - back.bytes;
        back.bytes = written;
        inner.stats.queued_bytes += grew;
        inner.stats.appended_records += 1;
        inner.stats.appended_bytes += plain_len;
        drop(inner);

        self.appended.notify_waiters();
        Ok(())
    }

    /// Close every open segment that has been collecting for long enough.
    ///
    /// Called by the sender before it looks for work: without it a quiet host
    /// would hold its records until a segment filled, which at eight mebibytes
    /// could be hours. Each signal keeps its own age, so a busy one does not
    /// carry a quiet one's records out with it any more.
    pub fn seal_if_due(&self) -> io::Result<()> {
        let _tag = memprof::enter(Arena::Queue);
        let mut inner = self.inner.lock();
        for signal in Signal::ALL {
            let due = inner.signals[signal.index()]
                .active_since
                .is_some_and(|since| since.elapsed() >= self.limits.max_segment_age);
            if due {
                self.roll(&mut inner, signal)?;
            }
        }
        Ok(())
    }

    /// Close every open segment whatever its age. What a clean shutdown does,
    /// so that the records they hold are on the device rather than left for
    /// the next process to recover.
    pub fn seal(&self) -> io::Result<()> {
        let _tag = memprof::enter(Arena::Queue);
        let mut inner = self.inner.lock();
        for signal in Signal::ALL {
            self.roll(&mut inner, signal)?;
        }
        Ok(())
    }

    /// The closed segment that has been waiting longest, whichever signal it
    /// belongs to. Everything on disk is still owed, so this is simply the
    /// oldest one that is not still being written.
    pub fn oldest_sealed(&self) -> Option<(Signal, u64)> {
        oldest_sealed(&self.inner.lock())
    }

    pub fn has_sealed(&self) -> bool {
        self.oldest_sealed().is_some()
    }

    pub async fn wait_for_sealed(&self) {
        loop {
            let notified = self.appended.notified();
            if self.has_sealed() {
                return;
            }
            notified.await;
        }
    }

    /// Read a closed segment into the bytes that go on the wire.
    ///
    /// The file is the body. Whether it decompresses is signy's to find out —
    /// a batch it cannot read is a `400`, which is a refusal, which drops the
    /// segment. Checking here would mean decompressing every segment twice to
    /// reach the same place.
    pub fn read_segment(&self, signal: Signal, seq: u64) -> io::Result<SealedSegment> {
        let _tag = memprof::enter(Arena::Send);
        let body = segment::map_whole(&self.dirs[signal.index()], seq)?;
        Ok(SealedSegment { signal, seq, body })
    }

    /// signy has every segment of this signal up to and including this one, so
    /// they are unlinked. That is the whole of the bookkeeping: nothing is
    /// written down about how far signy has got, because what is left on disk
    /// says it.
    pub fn commit(&self, signal: Signal, acked: u64) -> io::Result<()> {
        let mut inner = self.inner.lock();
        let dir = &self.dirs[signal.index()];
        let queue = &mut inner.signals[signal.index()];
        let active = queue
            .segments
            .back()
            .expect("a signal always holds its active segment")
            .seq;
        let mut freed = 0;
        while let Some(front) = queue.segments.front() {
            if front.seq > acked || front.seq == active {
                break;
            }
            let held = queue.segments.pop_front().expect("just inspected");
            segment::remove(dir, held.seq)?;
            freed += held.bytes;
        }
        inner.stats.queued_bytes -= freed;
        Ok(())
    }

    pub fn stats(&self) -> QueueStats {
        let inner = self.inner.lock();
        QueueStats {
            segments: inner.signals.iter().map(|queue| queue.segments.len()).sum(),
            ..inner.stats
        }
    }

    fn roll(&self, inner: &mut Inner, signal: Signal) -> io::Result<()> {
        // The file can still be empty while the encoder holds a block's worth,
        // so what says the segment is empty is that nothing was appended.
        if inner.signals[signal.index()].active_since.is_none() {
            return Ok(());
        }
        inner.stamped += 1;
        let stamp = inner.stamped;
        let queue = &mut inner.signals[signal.index()];
        let next = queue
            .segments
            .back()
            .expect("a signal always holds its active segment")
            .seq
            + 1;

        // Closing writes out what the encoder held and forces the lot to the
        // device, so the segment's true size is only known here. The
        // compressor comes back out with it and opens the next segment.
        let (bytes, encoder) = queue.active.finish()?;
        let back = queue
            .segments
            .back_mut()
            .expect("a signal always holds its active segment");
        let grew = bytes - back.bytes;
        back.bytes = bytes;
        back.stamp = stamp;

        queue.active = SegmentWriter::reusing(&self.dirs[signal.index()], next, encoder)?;
        queue.active_since = None;
        queue.segments.push_back(Held {
            seq: next,
            bytes: 0,
            stamp: 0,
        });
        inner.stats.queued_bytes += grew;
        self.appended.notify_waiters();
        Ok(())
    }

    fn drop_oldest(&self, inner: &mut Inner) -> io::Result<()> {
        if oldest_sealed(inner).is_none() {
            for signal in Signal::ALL {
                self.roll(inner, signal)?;
            }
        }
        let Some((signal, _)) = oldest_sealed(inner) else {
            // Three empty active segments, which cannot be rolled and cannot
            // be dropped.
            return Ok(());
        };
        let queue = &mut inner.signals[signal.index()];
        let held = queue.segments.pop_front().expect("just found sealed");
        segment::remove(&self.dirs[signal.index()], held.seq)?;
        inner.stats.queued_bytes -= held.bytes;
        inner.stats.dropped_bytes += held.bytes;
        inner.stats.dropped_segments += 1;
        Ok(())
    }
}

/// The oldest closed segment across the three signals, by the order they were
/// closed. A signal holding only its open segment has none.
fn oldest_sealed(inner: &Inner) -> Option<(Signal, u64)> {
    inner
        .signals
        .iter()
        .enumerate()
        .filter_map(|(slot, queue)| {
            let front = queue.segments.front()?;
            // The open segment is always the back one, so a single held
            // segment is the one still being written.
            (queue.segments.len() > 1).then_some((front.stamp, Signal::ALL[slot], front.seq))
        })
        .min_by_key(|(stamp, _, _)| *stamp)
        .map(|(_, signal, seq)| (signal, seq))
}
