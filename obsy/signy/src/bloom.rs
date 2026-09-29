use std::io::{self, Read, Write};

const MAGIC: &[u8; 4] = b"BLM1";

#[allow(dead_code)]
pub struct BloomFilter {
    bits: Vec<u8>,
    num_bits: usize,
    k: u32,
}

impl BloomFilter {
    pub fn new(num_bits: usize, k: u32) -> Self {
        assert!(
            num_bits > 0 && num_bits.is_multiple_of(8),
            "num_bits must be a positive multiple of 8"
        );
        Self {
            bits: vec![0; num_bits / 8],
            num_bits,
            k,
        }
    }

    pub fn with_capacity(estimated_items: usize, target_fpp: f64) -> Self {
        let num_bits = optimal_bits(estimated_items, target_fpp);
        let k = optimal_k(num_bits, estimated_items);
        Self::new(num_bits, k)
    }

    pub fn insert(&mut self, data: &[u8]) {
        let (h1, h2) = hash_pair(data);
        for i in 0..self.k {
            let idx = double_hash(h1, h2, i as u64, self.num_bits);
            self.bits[idx / 8] |= 1 << (idx % 8);
        }
    }

    pub fn contains(&self, data: &[u8]) -> bool {
        bits_contain(&self.bits, self.num_bits, self.k, data)
    }

    #[cfg(test)]
    pub fn insert_str_trigrams(&mut self, s: &str) {
        for tri in trigrams(s) {
            self.insert(&tri);
        }
    }

    pub fn might_contain_substr(&self, needle: &str) -> bool {
        bits_might_contain_substr(&self.bits, self.num_bits, self.k, needle)
    }

    #[allow(dead_code)]
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Bytes this filter holds in memory for as long as its part is open.
    pub fn resident_bytes(&self) -> usize {
        self.bits.len()
    }

    #[allow(dead_code)]
    pub fn k(&self) -> u32 {
        self.k
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.bits.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.num_bits as u32).to_le_bytes());
        out.extend_from_slice(&self.k.to_le_bytes());
        out.extend_from_slice(&self.bits);
        out
    }

    /// Number of bytes emitted by [`Self::write_encoded`].
    pub fn encoded_len(&self) -> usize {
        12 + self.bits.len()
    }

    /// Write the encoded filter without allocating a second buffer the size
    /// of the bitmap. The metric compactor keeps its output bounded, so a
    /// large filter must be streamed directly to its file.
    pub fn write_encoded<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(&(self.num_bits as u32).to_le_bytes())?;
        writer.write_all(&self.k.to_le_bytes())?;
        writer.write_all(&self.bits)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 12 {
            return Err("bloom buffer too short".to_string());
        }
        if &buf[0..4] != MAGIC {
            return Err("bloom magic mismatch".to_string());
        }
        let num_bits = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        let k = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if num_bits == 0 || !num_bits.is_multiple_of(8) {
            return Err(format!("invalid bloom num_bits {}", num_bits));
        }
        let bits_len = num_bits / 8;
        if buf.len() != 12 + bits_len {
            return Err(format!(
                "bloom length mismatch: expected {} got {}",
                12 + bits_len,
                buf.len()
            ));
        }
        Ok(Self {
            bits: buf[12..].to_vec(),
            num_bits,
            k,
        })
    }

    #[allow(dead_code)]
    pub fn write_to(&self, path: &std::path::Path) -> io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        f.write_all(&self.encode())?;
        f.sync_all()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read_from(path: &std::path::Path) -> Result<Self, String> {
        let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        Self::decode(&buf)
    }
}

/// The false-positive rate the line filter is sized for. A read-path number:
/// every point of it is scan work a `|=` cannot prune, so it is not a knob to
/// spend write capacity from.
pub const LINE_TRIGRAM_FPP: f64 = 0.01;

/// The distinct trigrams of a row group's lines, accumulated a line at a time
/// and turned into one filter sized to how many there were.
///
/// It is a type rather than a `lines -> filter` function so the write path can
/// keep collecting trigrams and exact-field tokens in **one** pass over the
/// rows, and so the structure underneath can be changed in one place — this is
/// the largest single term of `encode_group_blooms`, which the ceiling run put
/// at 63% of the flush pass.
///
/// **A bitmap over the whole domain, not a tree over what is present.** A
/// trigram is three bytes, so there are 2²⁴ of them and membership is one bit:
/// 2 MiB flat, `O(1)` per insert, against a `BTreeSet`'s `O(log n)` over
/// millions of inserts per part (a 100-byte line is ~98 trigrams and a part is
/// ~47,000 rows). The output is identical by construction — the same distinct
/// set, and a filter's bits do not depend on insertion order — which
/// `the_bitmap_and_a_tree_of_the_same_lines_build_the_same_filter` pins
/// against a reference implementation rather than against a golden blob.
///
/// The 2 MiB is transient, one row group at a time, and the flush path builds
/// groups sequentially: at most one of these is live per writer, so the
/// declared budget sees 2 MiB for flush and 2 MiB for a concurrent merge.
/// The same filter, borrowed rather than owned.
///
/// A stored bloom is a length, three header words and a bitset, laid out
/// contiguously in `index.bin`. Decoding it copied the bitset into a `Vec` —
/// and the copy is almost exactly the file: the soak measured a part's 4418
/// KiB of sidecar becoming 4120 KiB of decoded blooms, a factor of 1.01. So
/// the owned form bought nothing a slice of the mapping does not already
/// give, and cost a 4 MiB allocation per cache miss, at 200 misses a second
/// once the working set outgrew the cache.
///
/// [`BloomFilter`] stays as it is: the write path builds bits and needs to own
/// them. This is what the read path uses, made for the moment of a lookup out
/// of an offset into a mapping that outlives it, so nothing is cached but the
/// offsets.
#[derive(Clone, Copy)]
pub struct BloomRef<'a> {
    bits: &'a [u8],
    num_bits: usize,
    k: u32,
}

impl<'a> BloomRef<'a> {
    /// Reads the header a stored bloom begins with and borrows its bits.
    ///
    /// `buf` is the whole encoded filter — what [`BloomFilter::encode`] wrote
    /// and [`BloomFilter::decode`] parses. Returning the header rather than
    /// storing it lets a caller keep the three words and drop the bytes.
    pub fn parse(buf: &'a [u8]) -> Result<Self, String> {
        let (num_bits, k) = decode_bloom_header(buf)?;
        Ok(Self {
            bits: &buf[BLOOM_HEADER_BYTES..],
            num_bits,
            k,
        })
    }

    /// The same, from a header a caller already read and kept.
    pub fn from_parts(bits: &'a [u8], num_bits: usize, k: u32) -> Self {
        Self { bits, num_bits, k }
    }

    pub fn contains(&self, data: &[u8]) -> bool {
        bits_contain(self.bits, self.num_bits, self.k, data)
    }

    pub fn might_contain_substr(&self, needle: &str) -> bool {
        bits_might_contain_substr(self.bits, self.num_bits, self.k, needle)
    }
}

/// Bytes of header before the bitset: magic, `num_bits`, `k`.
pub const BLOOM_HEADER_BYTES: usize = 12;

/// Validates a stored bloom's header and returns `(num_bits, k)`.
///
/// Shared by the owning decoder and the borrowing one so a filter parsed
/// either way is rejected on exactly the same grounds.
pub fn decode_bloom_header(buf: &[u8]) -> Result<(usize, u32), String> {
    if buf.len() < BLOOM_HEADER_BYTES {
        return Err("bloom buffer too short".to_string());
    }
    if &buf[0..4] != MAGIC {
        return Err("bloom magic mismatch".to_string());
    }
    let num_bits = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    let k = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    if num_bits == 0 || !num_bits.is_multiple_of(8) {
        return Err(format!("invalid bloom num_bits {num_bits}"));
    }
    let bits_len = num_bits / 8;
    if buf.len() != BLOOM_HEADER_BYTES + bits_len {
        return Err(format!(
            "bloom length mismatch: expected {} got {}",
            BLOOM_HEADER_BYTES + bits_len,
            buf.len()
        ));
    }
    Ok((num_bits, k))
}

/// The lookup itself, over bits the caller owns or borrows.
fn bits_contain(bits: &[u8], num_bits: usize, k: u32, data: &[u8]) -> bool {
    let (h1, h2) = hash_pair(data);
    for i in 0..k {
        let idx = double_hash(h1, h2, i as u64, num_bits);
        if bits[idx / 8] & (1 << (idx % 8)) == 0 {
            return false;
        }
    }
    true
}

fn bits_might_contain_substr(bits: &[u8], num_bits: usize, k: u32, needle: &str) -> bool {
    let tris = trigrams(needle);
    if tris.is_empty() {
        return true;
    }
    tris.into_iter()
        .all(|tri| bits_contain(bits, num_bits, k, &tri))
}

pub struct TrigramSet {
    /// One bit per possible trigram, indexed big-endian so that iteration
    /// ascends in the same order a `BTreeSet<[u8; 3]>` would.
    words: Box<[u64]>,
    len: usize,
    /// The current line, lowercased, reused across every line of the group.
    /// [`trigrams`] returns a fresh `Vec` over a fresh `String` per call,
    /// which is two allocations per row — ~47,000 rows a part — for bytes
    /// that are dead before the next row.
    lowered: Vec<u8>,
}

/// 2²⁴ bits, one per trigram.
const TRIGRAM_WORDS: usize = (1 << 24) / 64;

impl Default for TrigramSet {
    fn default() -> Self {
        Self::new()
    }
}

impl TrigramSet {
    pub fn new() -> Self {
        Self {
            words: vec![0u64; TRIGRAM_WORDS].into_boxed_slice(),
            len: 0,
            lowered: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: &str) {
        let Self {
            words,
            len,
            lowered,
        } = self;
        lowercase_into(line, lowered);
        for window in lowered.windows(3) {
            let index = trigram_index([window[0], window[1], window[2]]);
            let bit = 1u64 << (index & 63);
            let word = &mut words[index >> 6];
            if *word & bit == 0 {
                *word |= bit;
                *len += 1;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// A filter sized to the distinct count, holding exactly those trigrams.
    /// The size is why the count has to be known before the first insert.
    pub fn finish(&self) -> BloomFilter {
        let mut bloom = BloomFilter::with_capacity(self.len().max(1), LINE_TRIGRAM_FPP);
        for (word_index, word) in self.words.iter().enumerate() {
            let mut bits = *word;
            while bits != 0 {
                let index = (word_index << 6) | bits.trailing_zeros() as usize;
                bloom.insert(&trigram_of(index));
                bits &= bits - 1;
            }
        }
        bloom
    }
}

/// `s.to_lowercase()`'s bytes, into a buffer the caller owns.
///
/// The ASCII branch is not an approximation of the general one: Unicode's
/// lowercase mapping of an ASCII scalar is its ASCII lowercase, so for an
/// all-ASCII line the two produce the same bytes — which is the whole of what
/// [`trigrams`] would have produced, and what the tree-versus-bitmap equality
/// tests compare against over generated corpora. It is worth the branch
/// because a log line is overwhelmingly ASCII and the general path walks
/// scalar by scalar through the case tables.
fn lowercase_into(s: &str, out: &mut Vec<u8>) {
    out.clear();
    if s.is_ascii() {
        out.extend_from_slice(s.as_bytes());
        out.make_ascii_lowercase();
        return;
    }
    let mut encoded = [0u8; 4];
    for lowered in s.chars().flat_map(char::to_lowercase) {
        out.extend_from_slice(lowered.encode_utf8(&mut encoded).as_bytes());
    }
}

fn trigram_index(trigram: [u8; 3]) -> usize {
    ((trigram[0] as usize) << 16) | ((trigram[1] as usize) << 8) | trigram[2] as usize
}

fn trigram_of(index: usize) -> [u8; 3] {
    [(index >> 16) as u8, (index >> 8) as u8, index as u8]
}

/// One row group's line filter, for callers that have the lines and nothing
/// else — the benches. The write path drives [`TrigramSet`] directly.
pub fn line_bloom<'a>(lines: impl Iterator<Item = &'a str>) -> BloomFilter {
    let mut set = TrigramSet::new();
    for line in lines {
        set.add_line(line);
    }
    set.finish()
}

pub fn trigrams(s: &str) -> Vec<[u8; 3]> {
    let lower = s.to_lowercase();
    let bytes = lower.as_bytes();
    let mut out = Vec::new();
    if bytes.len() < 3 {
        return out;
    }
    for i in 0..bytes.len() - 2 {
        out.push([bytes[i], bytes[i + 1], bytes[i + 2]]);
    }
    out
}

fn double_hash(h1: u64, h2: u64, i: u64, m: usize) -> usize {
    let combined = h1.wrapping_add(h2.wrapping_mul(i));
    (combined as usize) % m
}

fn hash_pair(data: &[u8]) -> (u64, u64) {
    (
        fnv1a_64(data, 0xcbf29ce484222325),
        fnv1a_64(data, 0x517cc1b727220a95),
    )
}

fn fnv1a_64(data: &[u8], seed: u64) -> u64 {
    let mut h = seed;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn optimal_bits(n: usize, p: f64) -> usize {
    if n == 0 {
        return 8192;
    }
    let m = (-(n as f64) * p.ln() / (std::f64::consts::LN_2.powi(2))).ceil() as usize;
    let m = m.max(1024);
    (m + 7) & !7
}

fn optimal_k(m: usize, n: usize) -> u32 {
    if n == 0 {
        return 4;
    }
    let k = ((m as f64 / n as f64) * std::f64::consts::LN_2).round() as u32;
    k.clamp(1, 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut b = BloomFilter::new(8192, 4);
        b.insert(b"hello world");
        assert!(b.contains(b"hello world"));
        assert!(!b.contains(b"goodbye world"));
    }

    #[test]
    fn trigram_substring_pruning() {
        let mut b = BloomFilter::with_capacity(100, 0.01);
        b.insert_str_trigrams("the quick brown fox");
        assert!(b.might_contain_substr("quick brown"));
        assert!(b.might_contain_substr("brown fox"));
        assert!(!b.might_contain_substr("zzzzzz"));
    }

    #[test]
    fn short_needle_skips_pruning() {
        let mut b = BloomFilter::new(8192, 4);
        b.insert_str_trigrams("abcdef");
        assert!(b.might_contain_substr("ab"));
    }

    #[test]
    fn encode_roundtrip() {
        let mut b = BloomFilter::with_capacity(200, 0.01);
        b.insert_str_trigrams("signy bloom filter test");
        let bytes = b.encode();
        let b2 = BloomFilter::decode(&bytes).expect("decode");
        assert_eq!(b.num_bits(), b2.num_bits());
        assert_eq!(b.k(), b2.k());
        assert!(b2.might_contain_substr("bloom filter"));
    }

    #[test]
    fn trigrams_basic() {
        let t = trigrams("abcd");
        assert_eq!(t, vec![*b"abc", *b"bcd"]);
        assert!(trigrams("ab").is_empty());
    }

    #[test]
    fn trigrams_lowercase() {
        let t = trigrams("ABC");
        assert_eq!(t, vec![*b"abc"]);
    }
}

#[cfg(test)]
mod trigram_set_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The structure this replaced, kept as the thing to be equal to. A golden
    /// byte blob would pin the filter to today's parameters; a reference
    /// implementation pins the only property that is actually claimed — that
    /// changing the *set* structure changed nothing about the *filter*.
    fn reference(lines: &[&str]) -> BloomFilter {
        let mut unique: BTreeSet<[u8; 3]> = BTreeSet::new();
        for line in lines {
            for trigram in trigrams(line) {
                unique.insert(trigram);
            }
        }
        let mut bloom = BloomFilter::with_capacity(unique.len().max(1), LINE_TRIGRAM_FPP);
        for trigram in &unique {
            bloom.insert(trigram);
        }
        bloom
    }

    fn corpus() -> Vec<&'static str> {
        vec![
            "GET /api/v1/query_range status=200 duration=13ms",
            "get /api/v1/query_range status=500 duration=1300ms",
            // Repeats, so the distinct count differs from the trigram count.
            "GET /api/v1/query_range status=200 duration=13ms",
            // Multi-byte, where a lowercase can change the byte length.
            "İstanbul ÅNGSTRÖM 三文字以上のログ行",
            // Shorter than a trigram: contributes nothing and must not panic.
            "ab",
            "",
            // The whole byte range, so no index is left unexercised by accident.
            "\u{0}\u{1}\u{2}\u{7f}\u{80}",
        ]
    }

    #[test]
    fn the_bitmap_and_a_tree_of_the_same_lines_build_the_same_filter() {
        let lines = corpus();
        let bitmap = line_bloom(lines.iter().copied());
        assert_eq!(
            bitmap.encode(),
            reference(&lines).encode(),
            "the trigram set's structure changed the filter it produces"
        );
    }

    /// The hand corpus above is chosen to be awkward; this one is the corpus
    /// the benches and the bed actually run, at a row group's scale. The
    /// −93% the swap measured is only a result if the filter it produces is
    /// the same filter, and "same" has to mean over real lines, not over
    /// seven.
    #[test]
    fn the_two_agree_over_a_whole_row_group_of_generated_lines() {
        for shape in [
            crate::corpus::Shape::Plain,
            crate::corpus::Shape::Json,
            crate::corpus::Shape::Logfmt,
        ] {
            let corpus = crate::corpus::generate(
                &crate::corpus::CorpusSpec::default()
                    .rows(4_000)
                    .streams(16)
                    .only(shape),
            );
            let lines = corpus.lines();
            let bitmap = line_bloom(lines.iter().copied());
            let tree = reference(&lines);
            assert_eq!(
                bitmap.encode(),
                tree.encode(),
                "{shape:?}: the bitmap and the tree disagree over a generated row group"
            );
            assert!(
                bitmap.num_bits() > 8192,
                "{shape:?}: a row group of real lines should not size to the floor"
            );
        }
    }

    #[test]
    fn the_distinct_count_is_the_filter_size_and_repeats_do_not_grow_it() {
        let mut once = TrigramSet::new();
        once.add_line("abcabc");
        let mut twice = TrigramSet::new();
        twice.add_line("abcabc");
        twice.add_line("abcabc");
        // "abc", "bca", "cab", "abc" -> three distinct.
        assert_eq!(once.len(), 3);
        assert_eq!(twice.len(), 3);
        assert_eq!(once.finish().num_bits(), twice.finish().num_bits());
    }

    #[test]
    fn a_line_shorter_than_a_trigram_adds_nothing() {
        let mut set = TrigramSet::new();
        set.add_line("ab");
        set.add_line("");
        assert!(set.is_empty());
    }

    #[test]
    fn every_trigram_index_round_trips_through_the_bitmap() {
        for trigram in [[0u8, 0, 0], [255, 255, 255], [0, 255, 0], [1, 2, 3]] {
            assert_eq!(trigram_of(trigram_index(trigram)), trigram);
        }
    }
}
