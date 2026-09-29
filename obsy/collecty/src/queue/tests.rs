use std::io::Write;
use std::sync::Arc;

use super::*;
use crate::signal::Signal;
use crate::test_support::Scratch;
use crate::wire::{self, ZSTD_LEVEL};

fn open(scratch: &Scratch, limits: QueueLimits) -> Queue {
    Queue::open(scratch.path(), limits, ZSTD_LEVEL).expect("a queue")
}

/// Segments close on age as well as size, and the sender is what asks. Tests
/// that want a closed segment ask the same way, with an age of nothing.
fn eager() -> QueueLimits {
    QueueLimits {
        max_segment_age: Duration::from_nanos(1),
        ..QueueLimits::default()
    }
}

/// A record written in pieces is the record written whole.
///
/// `route` hands the queue the frames hyper read rather than joining them
/// first, so the frame boundaries have to leave no trace: the length in front
/// is the total, and the pieces follow it in order.
#[test]
fn a_record_written_in_pieces_is_the_same_record() {
    let scratch = Scratch::new("append-pieces");
    let queue = open(&scratch, eager());
    let body: Vec<u8> = (0..5000u32).map(|byte| byte as u8).collect();

    queue.append(Signal::Logs, &body).expect("one write");
    let pieces: Vec<bytes::Bytes> = body
        .chunks(333)
        .map(bytes::Bytes::copy_from_slice)
        .collect();
    assert!(pieces.len() > 3, "the record has to arrive in pieces");
    queue
        .append_chunks(Signal::Logs, &pieces)
        .expect("many writes");

    assert_eq!(drain(&queue), vec![body.clone(), body]);
}

fn path_of(scratch: &Scratch, signal: Signal, seq: u64) -> std::path::PathBuf {
    scratch
        .path()
        .join(signal.as_str())
        .join(format!("{seq:020}.seg"))
}

/// A closed segment's records, as signy would read them: one zstd stream, and
/// behind it the records back to back.
fn records_of(sealed: &SealedSegment) -> Vec<Vec<u8>> {
    let plain = wire::decompress(&sealed.body, sealed.body.len() * 8).expect("a stream");
    wire::split_records(&plain)
        .expect("framed records")
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}

/// Every record collecty still holds, taken segment by segment the way the
/// sender takes them.
fn drain(queue: &Queue) -> Vec<Vec<u8>> {
    let mut records = Vec::new();
    queue.seal_if_due().expect("a seal");
    while let Some((signal, seq)) = queue.oldest_sealed() {
        let sealed = queue.read_segment(signal, seq).expect("a segment");
        records.extend(records_of(&sealed));
        queue.commit(signal, seq).expect("a commit");
        queue.seal_if_due().expect("a seal");
    }
    records
}

/// Bytes with nothing in them to compress, for tests about what a segment
/// occupies rather than about what it saves.
fn noise(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

fn lines(count: usize) -> Vec<Vec<u8>> {
    (0..count)
        .map(|index| format!("GET /v1/checkout 200 in 31ms request_id=req-{index:08}").into_bytes())
        .collect()
}

#[test]
fn a_closed_segment_is_one_stream_over_every_record_it_took() {
    let scratch = Scratch::new("stream");
    let queue = open(&scratch, eager());
    let bodies = lines(5);
    for body in &bodies {
        queue.append(Signal::Logs, body).expect("an append");
    }

    queue.seal_if_due().expect("a seal");
    let (signal, seq) = queue.oldest_sealed().expect("a closed segment");
    let sealed = queue.read_segment(signal, seq).expect("a segment");

    assert_eq!(records_of(&sealed), bodies);
    assert_eq!(
        sealed.body,
        std::fs::read(path_of(&scratch, signal, seq)).expect("the file"),
        "the file is the body, byte for byte"
    );
}

/// The whole point of one stream a segment: the second record costs almost
/// nothing once the first has taught the compressor what the data looks like.
#[test]
fn a_segment_compresses_across_its_records() {
    let scratch = Scratch::new("ratio");
    let queue = open(&scratch, eager());
    let bodies = lines(200);
    for body in &bodies {
        queue.append(Signal::Logs, body).expect("an append");
    }

    queue.seal_if_due().expect("a seal");
    let (signal, seq) = queue.oldest_sealed().expect("a closed segment");
    let together = queue
        .read_segment(signal, seq)
        .expect("a segment")
        .body
        .len();

    let apart: usize = bodies
        .iter()
        .map(|body| {
            zstd::bulk::compress(&wire::frame_record(body), ZSTD_LEVEL)
                .expect("a frame")
                .len()
        })
        .sum();

    assert!(
        together * 4 < apart,
        "one stream is {together} bytes against {apart} for a frame a record"
    );
}

/// The compressor outlives the segment, so the second segment is written by a
/// context that has already compressed the first. What it produces has to be
/// what a fresh context would have produced, or reuse would be a format change
/// dressed as an optimization.
#[test]
fn a_segment_written_by_a_reused_compressor_is_byte_for_byte_a_fresh_one() {
    let scratch = Scratch::new("reused-compressor");
    let queue = open(&scratch, eager());

    let first = lines(64);
    for body in &first {
        queue.append(Signal::Logs, body).expect("an append");
    }
    queue.seal_if_due().expect("a seal");

    // The same compressor now opens the second segment.
    let second = lines(64);
    for body in &second {
        queue.append(Signal::Logs, body).expect("an append");
    }
    queue.seal_if_due().expect("a seal");

    // A fresh *streaming* compressor, which is what the queue used to build
    // per segment. Not `zstd::bulk::compress`: that knows the whole input up
    // front and records its size in the frame header, which is a byte the
    // queue's stream never writes.
    let mut fresh = zstd::stream::write::Encoder::new(Vec::new(), ZSTD_LEVEL).expect("an encoder");
    for body in &second {
        std::io::Write::write_all(&mut fresh, &wire::frame_record(body)).expect("a frame");
    }
    let fresh = fresh.finish().expect("a stream");

    let written = queue.read_segment(Signal::Logs, 2).expect("a segment").body;
    assert_eq!(
        written,
        fresh,
        "a reused compressor wrote {} bytes where a fresh one writes {}",
        written.len(),
        fresh.len()
    );
}

/// The open segment is being appended to, so it is never offered. This is what
/// removes the one place a reader and a writer shared a file.
#[test]
fn the_open_segment_is_not_offered() {
    let scratch = Scratch::new("open");
    let queue = open(&scratch, QueueLimits::default());
    queue
        .append(Signal::Logs, b"still being written")
        .expect("an append");

    assert!(queue.oldest_sealed().is_none());
    assert!(!queue.has_sealed());
}

#[test]
fn an_open_segment_closes_once_it_is_old_enough() {
    let scratch = Scratch::new("age");
    let queue = open(
        &scratch,
        QueueLimits {
            max_segment_age: Duration::from_millis(30),
            ..QueueLimits::default()
        },
    );
    queue.append(Signal::Logs, b"waiting").expect("an append");

    queue.seal_if_due().expect("a seal");
    assert!(queue.oldest_sealed().is_none(), "not old enough yet");

    std::thread::sleep(Duration::from_millis(40));
    queue.seal_if_due().expect("a seal");
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 1)));
}

/// An empty queue must not roll segments forever while nothing arrives. The
/// file being empty is not the test: a segment that has taken records can
/// still be an empty file while the encoder holds them.
#[test]
fn an_empty_segment_is_never_closed() {
    let scratch = Scratch::new("idle");
    let queue = open(&scratch, eager());

    for _ in 0..5 {
        queue.seal_if_due().expect("a seal");
    }

    assert_eq!(
        queue.stats().segments,
        Signal::ALL.len(),
        "the open segment of each signal and nothing else"
    );
    assert!(queue.oldest_sealed().is_none());

    queue.append(Signal::Logs, b"arrived").expect("an append");
    queue.seal_if_due().expect("a seal");
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 1)));
}

#[test]
fn an_answered_segment_is_removed_and_not_offered_again() {
    let scratch = Scratch::new("commit");
    let queue = open(&scratch, eager());
    queue.append(Signal::Logs, b"first").expect("an append");
    queue.seal_if_due().expect("a seal");
    queue.append(Signal::Logs, b"second").expect("an append");
    queue.seal_if_due().expect("a seal");

    let (signal, seq) = queue.oldest_sealed().expect("a closed segment");
    assert_eq!((signal, seq), (Signal::Logs, 1));
    queue.commit(signal, seq).expect("a commit");

    assert!(
        !path_of(&scratch, Signal::Logs, 1).exists(),
        "the file is unlinked"
    );
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 2)));
}

#[test]
fn a_reopened_queue_keeps_what_signy_has_not_answered_for() {
    let scratch = Scratch::new("reopen");
    let sender = {
        let queue = open(&scratch, eager());
        queue.append(Signal::Logs, b"answered").expect("an append");
        queue.seal_if_due().expect("a seal");
        queue
            .append(Signal::Logs, b"still owed")
            .expect("an append");
        queue.seal_if_due().expect("a seal");
        queue.commit(Signal::Logs, 1).expect("a commit");
        queue.sender_id()
    };

    let queue = open(&scratch, eager());
    assert_eq!(queue.sender_id(), sender, "the id outlives the process");
    assert_eq!(drain(&queue), vec![b"still owed".to_vec()]);
}

/// A stream cannot be resumed, so what a previous process left open is closed
/// as it is and this one starts a segment of its own.
#[test]
fn a_reopened_queue_closes_the_old_segment_and_opens_a_new_one() {
    let scratch = Scratch::new("resume");
    {
        let queue = open(&scratch, QueueLimits::default());
        queue
            .append(Signal::Logs, b"before the restart")
            .expect("an append");
        queue.seal().expect("a seal on the way out");
    }

    let queue = open(&scratch, eager());
    assert_eq!(
        queue.oldest_sealed(),
        Some((Signal::Logs, 1)),
        "the old one is sendable"
    );
    queue.append(Signal::Logs, b"after it").expect("an append");
    assert_eq!(
        queue.oldest_sealed(),
        Some((Signal::Logs, 1)),
        "and not appended to"
    );
    assert_eq!(queue.stats().segments, Signal::ALL.len() + 1);

    assert_eq!(
        drain(&queue),
        vec![b"before the restart".to_vec(), b"after it".to_vec()]
    );
}

/// Nothing is written down about how far signy has got: an answered segment is
/// unlinked, so what is on disk is what is still owed. A crash between the
/// answer and the unlink leaves the file behind, and it is simply offered
/// again — signy answers that one without reading it.
#[test]
fn a_segment_left_behind_by_a_crash_is_offered_again() {
    let scratch = Scratch::new("stale");
    {
        let queue = open(&scratch, eager());
        queue.append(Signal::Logs, b"answered").expect("an append");
        queue.seal_if_due().expect("a seal");
    }
    assert!(path_of(&scratch, Signal::Logs, 1).exists());

    let queue = open(&scratch, eager());
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 1)));
    assert_eq!(drain(&queue), vec![b"answered".to_vec()]);
}

/// A segment closes and syncs in one step, so a process that dies without
/// closing loses whatever the encoder was still holding — and keeps every
/// record that had already reached the file.
#[test]
fn a_crash_keeps_the_records_that_reached_the_file() {
    let scratch = Scratch::new("crash");
    // Past the encoder's block size, so some of it is written and some is not.
    let bodies = lines(4000);
    {
        let queue = open(&scratch, QueueLimits::default());
        for body in &bodies {
            queue.append(Signal::Logs, body).expect("an append");
        }
    }

    let queue = open(&scratch, eager());
    let recovered = drain(&queue);

    assert!(!recovered.is_empty(), "what reached the file is kept");
    assert!(
        recovered.len() < bodies.len(),
        "and what the encoder still held is not"
    );
    assert_eq!(recovered, bodies[..recovered.len()]);
}

/// The stream ends wherever the crash left it, which is not a record boundary.
/// Recovery cuts back to one, because a segment ending mid-record is one signy
/// refuses whole.
#[test]
fn a_torn_tail_is_cut_back_to_the_last_whole_record() {
    let scratch = Scratch::new("torn");
    let bodies = lines(4000);
    {
        let queue = open(&scratch, QueueLimits::default());
        for body in &bodies {
            queue.append(Signal::Logs, body).expect("an append");
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path_of(&scratch, Signal::Logs, 1))
        .expect("the segment");
    file.write_all(&[9u8; 7]).expect("a torn write");
    drop(file);

    let queue = open(&scratch, eager());
    let recovered = drain(&queue);

    assert!(!recovered.is_empty());
    assert_eq!(recovered, bodies[..recovered.len()]);
}

/// Closing a recovered segment rewrites it, so what the sender picks up is a
/// stream that ends properly rather than one signy would refuse.
#[test]
fn a_recovered_segment_is_closed_before_it_is_offered() {
    let scratch = Scratch::new("recovered");
    {
        let queue = open(&scratch, QueueLimits::default());
        for body in lines(4000) {
            queue.append(Signal::Logs, &body).expect("an append");
        }
    }

    let queue = open(&scratch, eager());
    let sealed = queue.read_segment(Signal::Logs, 1).expect("a segment");
    wire::decompress(&sealed.body, sealed.body.len() * 8).expect("a stream that ends properly");
}

/// A segment that holds nothing readable leaves nothing behind.
#[test]
fn a_segment_that_decompresses_to_nothing_is_dropped() {
    let scratch = Scratch::new("garbage");
    {
        let queue = open(&scratch, QueueLimits::default());
        queue
            .append(Signal::Logs, b"never reached the file")
            .expect("an append");
    }
    assert_eq!(
        std::fs::metadata(path_of(&scratch, Signal::Logs, 1))
            .expect("a segment")
            .len(),
        0,
        "the encoder held all of it"
    );

    // The number is not reused. signy holds a high-water mark under this
    // sender's name, and numbering that went backwards would have it skip
    // every segment under that mark as one it already stored.
    let queue = open(&scratch, eager());
    assert!(drain(&queue).is_empty());
    queue.append(Signal::Logs, b"after it").expect("an append");
    queue.seal_if_due().expect("a seal");
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 2)));
}

#[test]
fn the_oldest_segment_is_dropped_when_the_queue_is_full() {
    let scratch = Scratch::new("drop");
    let queue = open(
        &scratch,
        QueueLimits {
            max_bytes: 4096,
            max_segment_bytes: 1024,
            ..eager()
        },
    );
    // A segment a record, so the cap has whole segments to unlink: what the
    // encoder still holds is not on disk and cannot be counted or dropped.
    // Bodies zstd cannot shrink, so the segments are the size they look.
    let bodies: Vec<Vec<u8>> = (0..40).map(|index| noise(index, 400)).collect();
    for body in &bodies {
        queue.append(Signal::Logs, body).expect("an append");
        queue.seal_if_due().expect("a seal");
    }

    let stats = queue.stats();
    assert!(stats.dropped_segments > 0);
    assert!(stats.dropped_bytes > 0);
    assert!(stats.queued_bytes <= 4096);

    let kept = drain(&queue);
    assert!(!kept.is_empty());
    assert_eq!(kept.last().expect("a record"), bodies.last().expect("one"));
}

/// Against the plain length, which is all that is known before the record has
/// been compressed.
#[test]
fn a_record_larger_than_the_whole_queue_is_refused() {
    let scratch = Scratch::new("oversized");
    let queue = open(
        &scratch,
        QueueLimits {
            max_bytes: 64,
            max_segment_bytes: 64,
            ..QueueLimits::default()
        },
    );
    let error = queue
        .append(Signal::Logs, &[0u8; 100])
        .expect_err("a refusal");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

/// A name that cannot be read is replaced rather than repaired. Keeping it
/// while the segments went back to one would have signy skip every segment
/// under the high-water mark it still held for that name.
#[test]
fn an_unreadable_identity_is_replaced() {
    let scratch = Scratch::new("identity");
    let before = {
        let queue = open(&scratch, eager());
        queue.append(Signal::Logs, b"first").expect("an append");
        queue.seal_if_due().expect("a seal");
        queue.commit(Signal::Logs, 1).expect("a commit");
        queue.sender_id()
    };

    std::fs::write(scratch.path().join("identity"), b"junk").expect("a damaged identity");

    let queue = open(&scratch, eager());
    assert_ne!(queue.sender_id(), before);
}

#[tokio::test]
async fn a_waiter_wakes_when_a_segment_closes() {
    let scratch = Scratch::new("wait");
    let queue = Arc::new(open(&scratch, eager()));
    let waiting = queue.clone();
    let waiter = tokio::spawn(async move { waiting.wait_for_sealed().await });

    tokio::time::sleep(Duration::from_millis(20)).await;
    queue.append(Signal::Logs, b"arrived").expect("an append");
    queue.seal_if_due().expect("a seal");

    tokio::time::timeout(Duration::from_secs(1), waiter)
        .await
        .expect("the waiter woke")
        .expect("the task finished");
}

/// What the queue occupies is what it still owes: an answered segment is gone
/// from disk, so there is no second number to keep.
#[test]
fn an_answered_segment_leaves_the_queue_smaller() {
    let scratch = Scratch::new("backlog");
    let queue = open(&scratch, eager());
    queue.append(Signal::Logs, &[1u8; 100]).expect("an append");
    queue.seal_if_due().expect("a seal");
    queue.append(Signal::Logs, &[2u8; 100]).expect("an append");
    queue.seal_if_due().expect("a seal");

    let before = queue.stats().queued_bytes;
    queue.commit(Signal::Logs, 1).expect("a commit");

    assert!(queue.stats().queued_bytes < before);
}

/// Plain bytes in, compressed bytes on disk. The two together are the ratio a
/// host is achieving, which is the reason they are counted differently.
#[test]
fn appended_bytes_are_what_arrived_and_queued_bytes_are_what_is_kept() {
    let scratch = Scratch::new("counts");
    let queue = open(&scratch, eager());
    let bodies = lines(200);
    for body in &bodies {
        queue.append(Signal::Logs, body).expect("an append");
    }
    queue.seal_if_due().expect("a seal");

    let stats = queue.stats();
    let plain: u64 = bodies
        .iter()
        .map(|body| (wire::RECORD_HEADER_BYTES + body.len()) as u64)
        .sum();
    assert_eq!(stats.appended_records, bodies.len() as u64);
    assert_eq!(stats.appended_bytes, plain);
    assert!(stats.queued_bytes < plain);
}

/// A segment holds one signal and nothing else, and each signal numbers its
/// own from one. Nothing on disk puts two signals' segments in one order,
/// which is why the wire carries the signal beside the number.
#[test]
fn a_segment_holds_one_signal_and_each_signal_numbers_its_own() {
    let scratch = Scratch::new("split");
    let queue = open(&scratch, eager());
    queue
        .append(Signal::Logs, b"a log line")
        .expect("an append");
    queue.append(Signal::Traces, b"a span").expect("an append");
    queue.seal_if_due().expect("a seal");

    let logs = queue.read_segment(Signal::Logs, 1).expect("a segment");
    let traces = queue.read_segment(Signal::Traces, 1).expect("a segment");
    assert_eq!(records_of(&logs), vec![b"a log line".to_vec()]);
    assert_eq!(records_of(&traces), vec![b"a span".to_vec()]);

    let mut closed = Vec::new();
    while let Some((signal, seq)) = queue.oldest_sealed() {
        closed.push((signal, seq));
        queue.commit(signal, seq).expect("a commit");
    }
    assert_eq!(
        closed,
        vec![(Signal::Logs, 1), (Signal::Traces, 1)],
        "a signal that took nothing closed nothing, and both start at one"
    );
}

/// Numbers only order a signal against itself, so what orders the three
/// against each other is the order they were created in.
#[test]
fn the_oldest_segment_goes_first_whichever_signal_it_is() {
    let scratch = Scratch::new("order");
    let queue = open(&scratch, eager());
    let order = [
        (Signal::Traces, 1),
        (Signal::Logs, 1),
        (Signal::Traces, 2),
        (Signal::Metrics, 1),
    ];
    for (signal, _) in order {
        queue.append(signal, b"one record").expect("an append");
        queue.seal_if_due().expect("a seal");
    }

    let mut sent = Vec::new();
    while let Some((signal, seq)) = queue.oldest_sealed() {
        sent.push((signal, seq));
        queue.commit(signal, seq).expect("a commit");
    }
    assert_eq!(sent, order);
}

/// The budget is the whole queue's rather than a share per signal, and what it
/// takes when it is exceeded is the segment that has been waiting longest —
/// which can belong to a signal other than the one being appended to.
#[test]
fn the_oldest_segment_is_dropped_whichever_signal_it_is() {
    let scratch = Scratch::new("drop-across");
    let queue = open(
        &scratch,
        QueueLimits {
            max_bytes: 2048,
            ..eager()
        },
    );
    // Bodies zstd cannot shrink, so the segments are the size they look.
    queue
        .append(Signal::Traces, &noise(1, 900))
        .expect("an append");
    queue.seal_if_due().expect("a seal");
    for index in 0..4 {
        queue
            .append(Signal::Logs, &noise(index + 2, 900))
            .expect("an append");
        queue.seal_if_due().expect("a seal");
    }

    assert!(
        !path_of(&scratch, Signal::Traces, 1).exists(),
        "the traces segment was the oldest, so it went first"
    );
    assert!(queue.stats().dropped_segments > 0);
    assert!(queue.stats().queued_bytes <= 2048);
}

/// The stamps that order the three signals are memory, and memory dies with
/// the process. What a restart has left is the files' own timestamps, and they
/// are enough to put them back in the order they were written.
#[test]
fn the_order_across_signals_survives_a_restart() {
    let scratch = Scratch::new("order-restart");
    {
        let queue = open(&scratch, eager());
        // A filesystem's timestamps are not infinitely fine, and two segments
        // written inside one tick are indistinguishable to a restart.
        for signal in [Signal::Metrics, Signal::Traces, Signal::Logs] {
            queue
                .append(signal, signal.as_str().as_bytes())
                .expect("an append");
            queue.seal_if_due().expect("a seal");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    let queue = open(&scratch, eager());
    let mut sent = Vec::new();
    while let Some((signal, seq)) = queue.oldest_sealed() {
        sent.push(signal);
        queue.commit(signal, seq).expect("a commit");
    }
    assert_eq!(
        sent,
        vec![Signal::Metrics, Signal::Traces, Signal::Logs],
        "oldest first, not signal order"
    );
}

/// A signal only closes a segment because its own has been collecting too
/// long. A busy one no longer carries a quiet one's records out with it.
#[test]
fn a_signal_closes_on_its_own_age() {
    let scratch = Scratch::new("age-apart");
    let queue = open(
        &scratch,
        QueueLimits {
            max_segment_age: Duration::from_millis(200),
            ..QueueLimits::default()
        },
    );
    queue
        .append(Signal::Logs, b"an early log")
        .expect("an append");
    std::thread::sleep(Duration::from_millis(500));
    queue
        .append(Signal::Traces, b"a late span")
        .expect("an append");

    queue.seal_if_due().expect("a seal");
    assert_eq!(queue.oldest_sealed(), Some((Signal::Logs, 1)));
    queue.commit(Signal::Logs, 1).expect("a commit");
    assert_eq!(
        queue.oldest_sealed(),
        None,
        "the span is not old enough to go yet"
    );
}
