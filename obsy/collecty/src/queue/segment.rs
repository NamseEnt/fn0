use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use bytes::Bytes;
use memmap2::Mmap;

const SEGMENT_SUFFIX: &str = ".seg";
const TEMPORARY_SUFFIX: &str = ".tmp";

/// A segment file as the directory describes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentFile {
    pub seq: u64,
    pub bytes: u64,
    /// When it was last written.
    ///
    /// Numbering runs per signal, so a number places a segment among its own
    /// signal's and nowhere else. This is the only thing left that puts three
    /// signals' segments in one order, and it is read once, at open, to
    /// recover the order this process then keeps in memory.
    pub modified: SystemTime,
}

/// The open segment: one zstd stream, written until the segment closes.
///
/// Records go in as plain bytes and the encoder decides when to emit a block,
/// so what the file holds trails what has been accepted. Nothing forces that
/// gap shut before the segment closes — `finish` is the only durability point
/// the queue has, and it is the same moment the segment becomes sendable.
///
/// The compressor outlives the segment. `finish` hands it back and the next
/// segment takes it: a zstd level 3 context is 3.5 MiB, it is allocated
/// through C `malloc` where neither the queue's accounting nor the process's
/// choice of Rust allocator reaches it, and a segment rolls as often as once
/// a second per signal.
pub struct SegmentWriter {
    inner: Option<Stream>,
}

type Stream = zstd::stream::zio::Writer<Counted, zstd::stream::raw::Encoder<'static>>;

/// The segment file, counting what the encoder has actually handed it.
///
/// This is the segment's size for every purpose the queue has: what the disk
/// holds and what the wire will carry are the same bytes.
struct Counted {
    file: File,
    written: u64,
}

impl Write for Counted {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let wrote = self.file.write(buf)?;
        self.written += wrote as u64;
        Ok(wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl SegmentWriter {
    /// A signal's first segment, which is the only one that builds a
    /// compressor.
    pub fn create(dir: &Path, seq: u64, level: i32) -> io::Result<SegmentWriter> {
        SegmentWriter::reusing(dir, seq, zstd::stream::raw::Encoder::new(level)?)
    }

    /// The next segment, on the compressor the last one finished with. The
    /// session is reset; the level and every other parameter are kept.
    pub fn reusing(
        dir: &Path,
        seq: u64,
        mut encoder: zstd::stream::raw::Encoder<'static>,
    ) -> io::Result<SegmentWriter> {
        use zstd::stream::raw::Operation;
        encoder.reinit()?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path(dir, seq))?;
        file.sync_all()?;
        sync_dir(dir)?;
        Ok(SegmentWriter {
            inner: Some(zstd::stream::zio::Writer::new(
                Counted { file, written: 0 },
                encoder,
            )),
        })
    }

    pub fn write_all(&mut self, plain: &[u8]) -> io::Result<()> {
        self.inner.as_mut().ok_or_else(closed)?.write_all(plain)
    }

    /// What the file holds. Behind what has been accepted by whatever the
    /// encoder is still holding, and caught up by `finish`.
    pub fn written(&self) -> u64 {
        self.inner
            .as_ref()
            .map(|stream| stream.writer().written)
            .unwrap_or(0)
    }

    /// Close the stream, force it to the device, and hand the compressor back
    /// for the next segment.
    pub fn finish(&mut self) -> io::Result<(u64, zstd::stream::raw::Encoder<'static>)> {
        let mut stream = self.inner.take().ok_or_else(closed)?;
        stream.finish()?;
        let (counted, encoder) = stream.into_inner();
        counted.file.sync_all()?;
        Ok((counted.written, encoder))
    }
}

pub fn open_for_read(dir: &Path, seq: u64) -> io::Result<File> {
    File::open(path(dir, seq))
}

/// A whole segment, mapped rather than read.
///
/// A sealed segment is the wire body byte for byte, so the file already is the
/// buffer and the send path can hand its pages to hyper untouched. Reading it
/// instead allocates the file's size on the heap and copies the page cache
/// into it, and a drain does that back to back as fast as the far end takes
/// segments. Those multi-megabyte transients are what a glibc arena ends up
/// sized by: `docs/MEMORY.md` measured 198 MB drained leaving +7.8 MiB
/// resident afterwards, against under 1 MiB of anything live.
///
/// The bytes move rather than disappear -- they are page cache now, which a
/// cgroup counts too -- so what this saves outright is the copy and the
/// allocation, and what it saves in the cage is whatever the kernel is willing
/// to reclaim. Both are `memory.current`'s to answer, not anon's.
///
/// An empty file cannot be mapped. A sealed segment is never empty, since zstd
/// writes a frame epilogue, but a crash can leave one behind, and answering
/// with no bytes retires it where an error would have the sender retry it
/// forever.
pub fn map_whole(dir: &Path, seq: u64) -> io::Result<Bytes> {
    let file = File::open(path(dir, seq))?;
    if file.metadata()?.len() == 0 {
        return Ok(Bytes::new());
    }
    // SAFETY: a sealed segment is immutable for as long as the queue holds it.
    // Nothing writes to it after `finish`, and the only other thing that ever
    // happens to it is being unlinked once signy has it, which leaves the
    // mapping intact. The directory is the queue's own.
    let map = unsafe { Mmap::map(&file)? };
    Ok(Bytes::from_owner(map))
}

pub fn remove(dir: &Path, seq: u64) -> io::Result<()> {
    std::fs::remove_file(path(dir, seq))
}

pub fn list(dir: &Path) -> io::Result<Vec<SegmentFile>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(digits) = name.strip_suffix(SEGMENT_SUFFIX) else {
            continue;
        };
        let Ok(seq) = digits.parse::<u64>() else {
            continue;
        };
        let metadata = entry.metadata()?;
        files.push(SegmentFile {
            seq,
            bytes: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }
    files.sort_by_key(|file| file.seq);
    Ok(files)
}

/// A rewrite that a crash interrupted. The segment it was repairing is still
/// there under its own name, so this is only a file to unlink.
pub fn sweep_temporaries(dir: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if name
            .to_str()
            .is_some_and(|name| name.ends_with(TEMPORARY_SUFFIX))
        {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// Close a segment that a previous process left open.
///
/// The encoder's state died with that process, so there is no appending to an
/// unfinished stream — the segment is closed where it stopped instead. What
/// decompresses is kept, cut back to the last record that arrived whole, and
/// written out again as a stream that ends properly. A segment ending mid
/// record is one signy refuses whole, which is why the cut is not left to it.
///
/// Returns the segment's size, or `None` if nothing in it survived and the
/// file is gone.
pub fn reseal(dir: &Path, seq: u64, level: i32) -> io::Result<Option<u64>> {
    let bytes = std::fs::metadata(path(dir, seq))?.len();
    if bytes == 0 {
        remove(dir, seq)?;
        sync_dir(dir)?;
        return Ok(None);
    }

    // An error here is the ordinary case, not a surprise: an unfinished stream
    // reads as far as its last complete block and then says so. What it read
    // before that is what a crash left behind.
    let mut plain = Vec::new();
    let whole = zstd::stream::read::Decoder::new(open_for_read(dir, seq)?)
        .and_then(|mut decoder| decoder.read_to_end(&mut plain))
        .is_ok();
    let kept = crate::wire::whole_records_len(&plain);

    if whole && kept == plain.len() {
        return Ok(Some(bytes));
    }
    tracing::warn!(
        segment = seq,
        bytes,
        recovered = kept,
        dropped = plain.len() - kept,
        "closing a segment a crash left open"
    );
    plain.truncate(kept);
    if plain.is_empty() {
        remove(dir, seq)?;
        sync_dir(dir)?;
        return Ok(None);
    }
    rewrite(dir, seq, &plain, level).map(Some)
}

fn rewrite(dir: &Path, seq: u64, plain: &[u8], level: i32) -> io::Result<u64> {
    let temporary = path(dir, seq).with_extension("tmp");
    let mut encoder = encoder(File::create(&temporary)?, level)?;
    encoder.write_all(plain)?;
    let counted = encoder.finish()?;
    counted.file.sync_all()?;
    std::fs::rename(&temporary, path(dir, seq))?;
    sync_dir(dir)?;
    Ok(counted.written)
}

fn encoder(file: File, level: i32) -> io::Result<zstd::stream::write::Encoder<'static, Counted>> {
    zstd::stream::write::Encoder::new(Counted { file, written: 0 }, level)
}

fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "the segment is already closed")
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

fn path(dir: &Path, seq: u64) -> PathBuf {
    dir.join(format!("{seq:020}{SEGMENT_SUFFIX}"))
}
