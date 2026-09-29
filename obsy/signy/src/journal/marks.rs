use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MARKS_FILE: &str = "journal.marks";
const SENDER_ID_BYTES: usize = 16;
const MARK_ENTRY_BYTES: usize = SENDER_ID_BYTES + 1 + 8 + 8 + 8;

/// How long a collecty may stay silent before its number is forgotten.
///
/// Forgetting one that is still alive costs duplicates, not loss: it comes
/// back as a sender this instance has never heard of, and everything it sends
/// is stored. Keeping every id that ever connected costs a file that only
/// grows, and a collecty whose disk is replaced leaves its old id behind every
/// time.
const FORGET_AFTER: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Which collecty a segment came from.
///
/// Random and made with the collector's queue directory, so it names the queue
/// rather than the host: a replaced volume arrives under a new id and its
/// segment numbering starts again without colliding with what is remembered
/// here.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SenderId([u8; SENDER_ID_BYTES]);

impl SenderId {
    pub fn parse(hex: &str) -> Option<SenderId> {
        if hex.len() != SENDER_ID_BYTES * 2 {
            return None;
        }
        let mut bytes = [0u8; SENDER_ID_BYTES];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok()?;
        }
        Some(SenderId(bytes))
    }
}

impl std::fmt::Display for SenderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for SenderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self}")
    }
}

/// Which of a collecty's three streams a segment belongs to.
///
/// A collecty keeps a queue per signal and numbers each of them from one, so a
/// sender id on its own does not place a segment. The pair does, and every
/// position here is held under the pair.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CollectSignal {
    Logs,
    Traces,
    Metrics,
}

impl CollectSignal {
    pub const ALL: [CollectSignal; 3] = [
        CollectSignal::Logs,
        CollectSignal::Traces,
        CollectSignal::Metrics,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CollectSignal::Logs => "logs",
            CollectSignal::Traces => "traces",
            CollectSignal::Metrics => "metrics",
        }
    }

    /// What the request names it. The wire spells the signal out rather than
    /// numbering it, so a request is readable in a log or a proxy.
    pub fn parse(name: &str) -> Option<CollectSignal> {
        CollectSignal::ALL
            .into_iter()
            .find(|signal| signal.as_str() == name)
    }

    /// What this instance's own records spell it as. A byte rather than a
    /// name, because it sits in every mark in the WAL and in every entry of
    /// the marks file.
    fn tag(self) -> u8 {
        match self {
            CollectSignal::Logs => 1,
            CollectSignal::Traces => 2,
            CollectSignal::Metrics => 3,
        }
    }

    fn from_tag(tag: u8) -> Option<CollectSignal> {
        CollectSignal::ALL
            .into_iter()
            .find(|signal| signal.tag() == tag)
    }
}

/// Where one of a sender's streams is up to: everything strictly before this
/// point is durable.
///
/// `segment` is the one being read and `records` how many of its records are
/// stored, counted from its first. A segment that arrived whole leaves
/// `(segment + 1, 0)` behind, which is what says it needs no part of it again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub segment: u64,
    pub records: u64,
}

impl Position {
    /// Segments are numbered from one, so this is "nothing of segment one
    /// yet" — below every position a collecty can send.
    pub const START: Position = Position {
        segment: 1,
        records: 0,
    };

    /// The last segment this sender has whole. Zero when it has none.
    pub fn whole_segments(self) -> u64 {
        self.segment.saturating_sub(1)
    }
}

/// One claim about one of a sender's streams, written into the WAL beside the
/// records it accounts for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectMark {
    pub sender: SenderId,
    pub signal: CollectSignal,
    pub at: Position,
}

/// How far each of a collecty's three streams has got, as far as this instance
/// is concerned.
///
/// A collecty sends a segment from its first record every time, so a resend
/// after a crash or a cut connection repeats whatever the earlier attempt got
/// through. The position here is where that attempt stopped; the records
/// before it are counted off and dropped rather than stored twice.
///
/// A signal is a stream of its own: its segments are numbered from one
/// independently of the other two, and they arrive interleaved with them, so
/// one position per sender would have each signal walking the others' back.
///
/// The position is written by the journal writer **in the same batch as the
/// records it covers**, so one `sync_all` makes both durable together. There
/// is no window in which a record is on disk and the mark that would skip its
/// twin is not.
#[derive(Default)]
pub struct CollectMarks {
    inner: std::sync::RwLock<HashMap<(SenderId, CollectSignal), Seen>>,
}

#[derive(Clone, Copy)]
struct Seen {
    at: Position,
    when: SystemTime,
}

impl CollectMarks {
    /// Where this sender's stream for this signal is up to. The start of
    /// segment one for one never heard from, which is below everything it can
    /// send, so nothing is skipped.
    pub fn position(&self, sender: &SenderId, signal: CollectSignal) -> Position {
        self.inner
            .read()
            .expect("collect marks lock")
            .get(&(*sender, signal))
            .map(|seen| seen.at)
            .unwrap_or(Position::START)
    }

    /// Only ever forward. A mark that arrives out of order, or a resend of one
    /// already covered, must not walk the position back over records that
    /// would then be stored a second time.
    pub fn advance(&self, mark: CollectMark) {
        let mut inner = self.inner.write().expect("collect marks lock");
        let seen = inner.entry((mark.sender, mark.signal)).or_insert(Seen {
            at: Position::START,
            when: UNIX_EPOCH,
        });
        seen.at = seen.at.max(mark.at);
        seen.when = SystemTime::now();
    }

    /// How many streams are remembered — up to three per collecty.
    pub fn streams(&self) -> usize {
        self.inner.read().expect("collect marks lock").len()
    }

    /// Read what the last checkpoint knew.
    ///
    /// Anything unreadable is nothing: the file only ever makes the marks more
    /// complete than the WAL suffix alone, so losing it costs duplicates after
    /// a restart rather than correctness.
    pub fn load(dir: &std::path::Path) -> CollectMarks {
        let marks = CollectMarks::default();
        let Ok(bytes) = std::fs::read(dir.join(MARKS_FILE)) else {
            return marks;
        };
        if bytes.len() < 8 {
            return marks;
        }
        let body = bytes.len() - 4;
        let stored = u32::from_le_bytes(bytes[body..].try_into().expect("four bytes"));
        if crc32fast::hash(&bytes[..body]) != stored {
            tracing::warn!("journal marks failed their checksum; starting from the WAL alone");
            return marks;
        }
        let count = u32::from_le_bytes(bytes[0..4].try_into().expect("four bytes")) as usize;
        if body != 4 + count * MARK_ENTRY_BYTES {
            return marks;
        }
        let mut inner = marks.inner.write().expect("collect marks lock");
        for index in 0..count {
            let at = 4 + index * MARK_ENTRY_BYTES;
            let mut id = [0u8; SENDER_ID_BYTES];
            id.copy_from_slice(&bytes[at..at + SENDER_ID_BYTES]);
            let Some(signal) = CollectSignal::from_tag(bytes[at + SENDER_ID_BYTES]) else {
                continue;
            };
            let number = |from: usize| {
                u64::from_le_bytes(bytes[from..from + 8].try_into().expect("eight bytes"))
            };
            let segment = number(at + SENDER_ID_BYTES + 1);
            let records = number(at + SENDER_ID_BYTES + 9);
            let seconds = number(at + SENDER_ID_BYTES + 17);
            inner.insert(
                (SenderId(id), signal),
                Seen {
                    at: Position { segment, records },
                    when: UNIX_EPOCH + Duration::from_secs(seconds),
                },
            );
        }
        drop(inner);
        marks
    }

    /// Write the marks out, forgetting whoever has been silent too long.
    ///
    /// Written **before** the checkpoint offset that retires the WAL prefix
    /// these marks came from. The two are separate files and a crash can land
    /// between them, and this is the harmless order: marks ahead of the
    /// checkpoint name records the WAL still holds and replay will put back,
    /// while marks behind it would have those records sent again.
    pub fn store(&self, dir: &std::path::Path) -> Result<(), std::io::Error> {
        let now = SystemTime::now();
        let live: Vec<((SenderId, CollectSignal), Seen)> = {
            let mut inner = self.inner.write().expect("collect marks lock");
            inner.retain(|_, seen| {
                now.duration_since(seen.when)
                    .map(|silent| silent < FORGET_AFTER)
                    .unwrap_or(true)
            });
            inner.iter().map(|(key, seen)| (*key, *seen)).collect()
        };

        let mut bytes = Vec::with_capacity(8 + live.len() * MARK_ENTRY_BYTES);
        bytes.extend_from_slice(&(live.len() as u32).to_le_bytes());
        for ((id, signal), seen) in &live {
            bytes.extend_from_slice(&id.0);
            bytes.push(signal.tag());
            bytes.extend_from_slice(&seen.at.segment.to_le_bytes());
            bytes.extend_from_slice(&seen.at.records.to_le_bytes());
            let seconds = seen
                .when
                .duration_since(UNIX_EPOCH)
                .map(|since| since.as_secs())
                .unwrap_or(0);
            bytes.extend_from_slice(&seconds.to_le_bytes());
        }
        let crc = crc32fast::hash(&bytes);
        bytes.extend_from_slice(&crc.to_le_bytes());

        let path = dir.join(MARKS_FILE);
        let tmp = path.with_extension("marks.tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::File::open(&tmp)?.sync_all()?;
        std::fs::rename(&tmp, &path)?;
        std::fs::File::open(dir)?.sync_all()?;
        Ok(())
    }
}

/// `LGYM | sender | signal | segment | records`, a WAL record of its own rather
/// than a field on the records it covers.
///
/// One per sender per batch, not one per record: the writer already groups
/// what is in its channel into a single write and a single `sync_all`, and the
/// mark rides in that group. Putting the position on every record instead
/// would have paid for it once per export to say something the batch says
/// once.
const MARK_RECORD_MAGIC: &[u8; 4] = b"LGYM";
pub const MARK_RECORD_BYTES: usize = 4 + SENDER_ID_BYTES + 1 + 8 + 8;

pub fn frame_mark(mark: &CollectMark, into: &mut Vec<u8>) {
    into.extend_from_slice(MARK_RECORD_MAGIC);
    into.extend_from_slice(&mark.sender.0);
    into.push(mark.signal.tag());
    into.extend_from_slice(&mark.at.segment.to_le_bytes());
    into.extend_from_slice(&mark.at.records.to_le_bytes());
}

pub fn decode_mark(data: &[u8]) -> Option<CollectMark> {
    if data.len() != MARK_RECORD_BYTES || !data.starts_with(MARK_RECORD_MAGIC) {
        return None;
    }
    let mut id = [0u8; SENDER_ID_BYTES];
    id.copy_from_slice(&data[4..4 + SENDER_ID_BYTES]);
    let signal = CollectSignal::from_tag(data[4 + SENDER_ID_BYTES])?;
    let number =
        |from: usize| u64::from_le_bytes(data[from..from + 8].try_into().expect("eight bytes"));
    Some(CollectMark {
        sender: SenderId(id),
        signal,
        at: Position {
            segment: number(4 + SENDER_ID_BYTES + 1),
            records: number(4 + SENDER_ID_BYTES + 9),
        },
    })
}
