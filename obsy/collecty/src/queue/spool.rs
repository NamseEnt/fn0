//! The one thread that touches the queue's files.
//!
//! `Queue::append` takes the queue's lock as its first act, so every append is
//! serialized whichever thread runs it. Handing each one to the blocking pool
//! therefore bought no parallelism at all, and cost a thread per concurrent
//! export -- tokio will spawn up to 512 of them. Under glibc a thread that
//! allocates is given an arena of its own, and an arena keeps whatever it grew
//! to: `docs/MEMORY.md` measured that term at three quarters of the footprint
//! by taking it away with `MALLOC_ARENA_MAX=1`.
//!
//! So the appends went to one thread that owns the queue, which is what the
//! lock already made them. Everything else the queue does to a file has now
//! followed them here: closing a segment and `fsync`ing it, opening and
//! mapping one to send, and unlinking the ones signy has answered for. None of
//! those is asynchronous -- an `fsync` is a syscall that returns when the
//! device says so -- and every one of them used to run on a tokio worker,
//! where a stall is not one task waiting but every task on that worker
//! waiting. That is survivable with a worker per core and it is not survivable
//! with one.
//!
//! There are two threads and not one, and which work goes where is decided by
//! what was already serial. `append`, `seal` and `commit` all take the queue's
//! lock, so they waited for each other before this module existed and putting
//! them in one queue changes nothing. `read_segment` takes no lock: it opens a
//! sealed file and maps it, and it ran beside an append happily.
//!
//! Putting the reads in with the writes was measured and undone. A drain runs
//! reads back to back at whatever rate the far end takes them, so ingest ended
//! up behind them in one FIFO -- and not because the FIFO filled, which never
//! got past 10 of 256, but because one slow file operation at the head makes
//! every short append behind it wait. In the draining window that took exports
//! over 100 ms from 0.02 % to 0.23 % and produced the first ones over 500 ms
//! this rig has seen. So reads have a lane of their own, which is the
//! concurrency the collector had before any of this, and `SpoolReport` says
//! what each lane waited.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::{fmt, io};

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

use super::{Queue, SealedSegment};
use crate::memprof::{self, Arena};
use crate::signal::Signal;

/// Jobs that may be waiting for the spool thread. Deep enough that a burst
/// does not serialize on the channel and shallow enough to be nothing: the
/// bytes are already bounded by the in-flight gate.
const DEPTH: usize = 256;

/// Wait times, in powers of two microseconds. A histogram and not a mean
/// because the question this answers is what happened while an `fsync` was
/// stalling, which a mean is exactly the wrong shape to show.
const BUCKETS: usize = 33;

/// What one write asked for, and where the answer goes.
enum Work {
    Append {
        signal: Signal,
        chunks: Vec<Bytes>,
        reply: oneshot::Sender<io::Result<()>>,
    },
    SealIfDue {
        reply: oneshot::Sender<io::Result<()>>,
    },
    Seal {
        reply: oneshot::Sender<io::Result<()>>,
    },
    Commit {
        signal: Signal,
        acked: u64,
        reply: oneshot::Sender<io::Result<()>>,
    },
}

struct Job {
    /// When it was handed over, so the thread can say how long it sat.
    at: Instant,
    work: Work,
}

/// One segment to open and map. Its own type because it is its own lane.
struct ReadJob {
    at: Instant,
    signal: Signal,
    seq: u64,
    reply: oneshot::Sender<io::Result<SealedSegment>>,
}

struct Counters {
    max_depth: AtomicU64,
    requests: AtomicU64,
    waits: [AtomicU64; BUCKETS],
}

impl Default for Counters {
    fn default() -> Counters {
        Counters {
            max_depth: AtomicU64::new(0),
            requests: AtomicU64::new(0),
            waits: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

/// How one lane is keeping up.
#[derive(Clone, Copy, Debug, Default)]
pub struct LaneReport {
    pub requests: u64,
    /// The most that were ever waiting at once, out of [`DEPTH`].
    pub max_depth: u64,
    /// Since the process started, so a run's last report covers the run.
    pub wait_p99_us: u64,
    pub wait_max_us: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SpoolReport {
    pub writes: LaneReport,
    pub reads: LaneReport,
}

/// The handle every caller holds. Cloning it is cloning two channel senders.
#[derive(Clone)]
pub struct Spool {
    jobs: mpsc::Sender<Job>,
    counters: Arc<Counters>,
    reads: mpsc::Sender<ReadJob>,
    read_counters: Arc<Counters>,
}

/// The spool thread is gone, which only happens on the way out.
#[derive(Debug)]
pub struct Gone;

impl fmt::Display for Gone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the spool thread stopped")
    }
}

impl Spool {
    pub fn new(queue: Arc<Queue>) -> Spool {
        let (jobs, mut inbox) = mpsc::channel::<Job>(DEPTH);
        let counters = Arc::new(Counters::default());
        let theirs = counters.clone();
        let reading = queue.clone();
        std::thread::Builder::new()
            .name("spool".to_string())
            .spawn(move || {
                while let Some(job) = inbox.blocking_recv() {
                    theirs.waited(job.at.elapsed().as_micros());
                    match job.work {
                        Work::Append {
                            signal,
                            chunks,
                            reply,
                        } => {
                            let _tag = memprof::enter(Arena::Intake);
                            // A caller that has gone away is a client that
                            // hung up between the append and the answer. The
                            // record is in the segment either way.
                            let _ = reply.send(queue.append_chunks(signal, &chunks));
                        }
                        Work::SealIfDue { reply } => {
                            let _ = reply.send(queue.seal_if_due());
                        }
                        Work::Seal { reply } => {
                            let _ = reply.send(queue.seal());
                        }
                        Work::Commit {
                            signal,
                            acked,
                            reply,
                        } => {
                            let _ = reply.send(queue.commit(signal, acked));
                        }
                    }
                }
            })
            .expect("the spool thread starts");

        let (reads, mut inbox) = mpsc::channel::<ReadJob>(DEPTH);
        let read_counters = Arc::new(Counters::default());
        let theirs = read_counters.clone();
        std::thread::Builder::new()
            .name("reader".to_string())
            .spawn(move || {
                while let Some(job) = inbox.blocking_recv() {
                    theirs.waited(job.at.elapsed().as_micros());
                    let _ = job.reply.send(reading.read_segment(job.signal, job.seq));
                }
            })
            .expect("the reader thread starts");

        Spool {
            jobs,
            counters,
            reads,
            read_counters,
        }
    }

    pub fn report(&self) -> SpoolReport {
        SpoolReport {
            writes: self.counters.report(),
            reads: self.read_counters.report(),
        }
    }

    /// The export as it arrived, in the pieces it arrived in.
    pub async fn append(&self, signal: Signal, chunks: Vec<Bytes>) -> Result<io::Result<()>, Gone> {
        self.ask(|reply| Work::Append {
            signal,
            chunks,
            reply,
        })
        .await
    }

    pub async fn seal_if_due(&self) -> Result<io::Result<()>, Gone> {
        self.ask(|reply| Work::SealIfDue { reply }).await
    }

    pub async fn seal(&self) -> Result<io::Result<()>, Gone> {
        self.ask(|reply| Work::Seal { reply }).await
    }

    /// On the reader's lane, so a drain reading segments back to back does not
    /// put every append behind it. Nothing here takes the queue's lock.
    pub async fn read_segment(
        &self,
        signal: Signal,
        seq: u64,
    ) -> Result<io::Result<SealedSegment>, Gone> {
        let (reply, answer) = oneshot::channel();
        self.read_counters
            .depth((DEPTH - self.reads.capacity()) as u64 + 1);
        self.reads
            .send(ReadJob {
                at: Instant::now(),
                signal,
                seq,
                reply,
            })
            .await
            .map_err(|_| Gone)?;
        answer.await.map_err(|_| Gone)
    }

    pub async fn commit(&self, signal: Signal, acked: u64) -> Result<io::Result<()>, Gone> {
        self.ask(|reply| Work::Commit {
            signal,
            acked,
            reply,
        })
        .await
    }

    /// Hand over one job and wait for its answer.
    ///
    /// The depth is read before the send rather than after, so a burst that
    /// fills the channel is counted at the moment it does.
    async fn ask<T>(
        &self,
        work: impl FnOnce(oneshot::Sender<io::Result<T>>) -> Work,
    ) -> Result<io::Result<T>, Gone> {
        let (reply, answer) = oneshot::channel();
        self.counters
            .depth((DEPTH - self.jobs.capacity()) as u64 + 1);
        self.jobs
            .send(Job {
                at: Instant::now(),
                work: work(reply),
            })
            .await
            .map_err(|_| Gone)?;
        answer.await.map_err(|_| Gone)
    }
}

impl Counters {
    fn depth(&self, now: u64) {
        self.max_depth.fetch_max(now, Ordering::Relaxed);
    }

    fn waited(&self, micros: u128) {
        let micros = micros.min(u32::MAX as u128).max(1) as u32;
        let bucket = (32 - micros.leading_zeros()) as usize;
        self.waits[bucket.min(BUCKETS - 1)].fetch_add(1, Ordering::Relaxed);
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self) -> LaneReport {
        let counts: Vec<u64> = self
            .waits
            .iter()
            .map(|bucket| bucket.load(Ordering::Relaxed))
            .collect();
        let total: u64 = counts.iter().sum();
        let mut p99 = 0;
        let mut max = 0;
        let mut seen = 0;
        for (index, count) in counts.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            let upper = 1u64 << index;
            max = upper;
            seen += count;
            if p99 == 0 && total > 0 && seen * 100 >= total * 99 {
                p99 = upper;
            }
        }
        LaneReport {
            requests: self.requests.load(Ordering::Relaxed),
            max_depth: self.max_depth.load(Ordering::Relaxed),
            wait_p99_us: p99,
            wait_max_us: max,
        }
    }
}
