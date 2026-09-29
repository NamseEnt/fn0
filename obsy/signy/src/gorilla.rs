//! Gorilla sample compression: delta-of-delta timestamps, XOR values.
//!
//! The published design (Pelkonen et al., "Gorilla: A Fast, Scalable,
//! In-Memory Time Series Database", VLDB 2015), implemented for this engine's
//! one sample kind — `(timestamp_ns: i64, value: f64)` — with the bucket
//! widths Prometheus's TSDB settled on for the delta-of-delta. Nothing here
//! is tuned past the paper: sample encoding is an axis where parity is the
//! bar (issue #8), and the entropy limit is close enough to the paper's
//! layout that cleverness would buy risk, not bytes.
//!
//! One chunk holds one series' samples in **append order**. The encoder does
//! not sort and does not reject a backwards timestamp — the delta-of-delta is
//! signed — because ordering is the caller's contract: the memtable appends
//! in-order samples here and diverts out-of-order arrivals to its spill
//! vector, and the flush merge-sorts the two. A chunk is therefore sorted
//! exactly when its writer honored that contract, which the tests pin.
//!
//! Chunk layout: `u32 LE sample count`, then the bitstream. The count rides
//! in the chunk rather than beside it so a chunk is self-describing wherever
//! it lands (memtable, part, journal-less abort path).

use std::borrow::Cow;

/// Bit-packed writer. Bits fill each byte from the high end, the direction
/// every Gorilla description assumes.
#[derive(Clone)]
struct BitWriter {
    bytes: Vec<u8>,
    /// Free bits remaining in the final byte, 0..8. 0 means the last byte is
    /// full (or the buffer is empty).
    free: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            free: 0,
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if self.free == 0 {
            self.bytes.push(0);
            self.free = 8;
        }
        if bit {
            let last = self.bytes.last_mut().expect("a byte was just ensured");
            *last |= 1 << (self.free - 1);
        }
        self.free -= 1;
    }

    /// The low `count` bits of `value`, most significant first.
    fn write_bits(&mut self, value: u64, count: u8) {
        for index in (0..count).rev() {
            self.write_bit((value >> index) & 1 == 1);
        }
    }
}

/// Bit-packed reader over a chunk's bitstream.
struct BitReader<'a> {
    bytes: Cow<'a, [u8]>,
    position: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            // The first four bytes are the decoder's sample-count header.
            position: 32,
        }
    }

    fn owned(bytes: Vec<u8>) -> BitReader<'static> {
        BitReader {
            bytes: Cow::Owned(bytes),
            position: 32,
        }
    }

    fn read_bit(&mut self) -> Result<bool, String> {
        let byte = self
            .bytes
            .get(self.position / 8)
            .ok_or_else(|| "gorilla chunk ends mid-record".to_string())?;
        let bit = (byte >> (7 - (self.position % 8))) & 1 == 1;
        self.position += 1;
        Ok(bit)
    }

    fn read_bits(&mut self, count: u8) -> Result<u64, String> {
        let mut value = 0u64;
        for _ in 0..count {
            value = (value << 1) | u64::from(self.read_bit()?);
        }
        Ok(value)
    }
}

fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

fn unzigzag(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

/// The delta-of-delta buckets, offset-encoded: `('0')` zero,
/// `('10', 14 bits)` for −8191..=8192, `('110', 17 bits)` for
/// −65535..=65536, `('1110', 20 bits)` for −524287..=524288, and
/// `('1111', 64 bits zigzag)` for everything else. Each row is
/// `(prefix bits, prefix length, payload bits, offset)`.
const DOD_BUCKETS: [(u64, u8, u8, i64); 3] = [
    (0b10, 2, 14, 8191),
    (0b110, 3, 17, 65535),
    (0b1110, 4, 20, 524287),
];

#[derive(Clone)]
pub struct Encoder {
    writer: BitWriter,
    count: u32,
    prev_ts: i64,
    prev_delta: i64,
    prev_bits: u64,
    leading: u8,
    trailing: u8,
    /// Whether a value has established the leading/trailing window yet.
    window_set: bool,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            writer: BitWriter::new(),
            count: 0,
            prev_ts: 0,
            prev_delta: 0,
            prev_bits: 0,
            leading: 0,
            trailing: 0,
            window_set: false,
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Encoded bytes held so far, for the memtable's self-metering. The
    /// 4-byte count header is included so the estimate equals what `close`
    /// returns.
    pub fn byte_len(&self) -> usize {
        4 + self.writer.bytes.len()
    }

    pub fn append(&mut self, ts_ns: i64, value: f64) {
        match self.count {
            0 => {
                self.writer.write_bits(ts_ns as u64, 64);
                self.writer.write_bits(value.to_bits(), 64);
            }
            1 => {
                let delta = ts_ns.wrapping_sub(self.prev_ts);
                self.writer.write_bits(zigzag(delta), 64);
                self.prev_delta = delta;
                self.append_value(value);
            }
            _ => {
                let delta = ts_ns.wrapping_sub(self.prev_ts);
                let dod = delta.wrapping_sub(self.prev_delta);
                if dod == 0 {
                    self.writer.write_bit(false);
                } else {
                    let mut written = false;
                    for (prefix, prefix_bits, payload_bits, offset) in DOD_BUCKETS {
                        // Each bucket holds -offset ..= offset + 1, the
                        // asymmetric ranges the TSDB layout uses so the offset
                        // encoding fills the field exactly.
                        if (-offset..=offset + 1).contains(&dod) {
                            self.writer.write_bits(prefix, prefix_bits);
                            self.writer.write_bits((dod + offset) as u64, payload_bits);
                            written = true;
                            break;
                        }
                    }
                    if !written {
                        self.writer.write_bits(0b1111, 4);
                        self.writer.write_bits(zigzag(dod), 64);
                    }
                }
                self.prev_delta = delta;
                self.append_value(value);
            }
        }
        self.prev_ts = ts_ns;
        self.prev_bits = value.to_bits();
        self.count += 1;
    }

    fn append_value(&mut self, value: f64) {
        let bits = value.to_bits();
        let xor = bits ^ self.prev_bits;
        if xor == 0 {
            self.writer.write_bit(false);
            return;
        }
        self.writer.write_bit(true);
        // Leading capped at 31 so it fits the 5-bit field; a value with more
        // leading zeros just writes a few extra meaningful bits.
        let leading = (xor.leading_zeros() as u8).min(31);
        let trailing = xor.trailing_zeros() as u8;
        if self.window_set && leading >= self.leading && trailing >= self.trailing {
            self.writer.write_bit(false);
            let significant = 64 - self.leading - self.trailing;
            self.writer.write_bits(xor >> self.trailing, significant);
        } else {
            self.writer.write_bit(true);
            let significant = 64 - leading - trailing;
            self.writer.write_bits(u64::from(leading), 5);
            // 6 bits hold 1..=64 with 64 written as 0.
            self.writer.write_bits(u64::from(significant) & 63, 6);
            self.writer.write_bits(xor >> trailing, significant);
            self.leading = leading;
            self.trailing = trailing;
            self.window_set = true;
        }
    }

    /// The chunk as stored: count header plus bitstream. The encoder is
    /// consumed — a Gorilla stream cannot be appended to after its trailing
    /// byte padding exists.
    pub fn close(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.writer.bytes.len());
        bytes.extend_from_slice(&self.count.to_le_bytes());
        bytes.extend_from_slice(&self.writer.bytes);
        bytes
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Decoder<'a> {
    reader: BitReader<'a>,
    remaining: u32,
    total: u32,
    prev_ts: i64,
    prev_delta: i64,
    prev_bits: u64,
    leading: u8,
    trailing: u8,
}

impl<'a> Decoder<'a> {
    pub fn new(chunk: &'a [u8]) -> Result<Self, String> {
        if chunk.len() < 4 {
            return Err("gorilla chunk is shorter than its count header".to_string());
        }
        let count = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        Ok(Self {
            reader: BitReader::new(chunk),
            remaining: count,
            total: count,
            prev_ts: 0,
            prev_delta: 0,
            prev_bits: 0,
            leading: 0,
            trailing: 0,
        })
    }

    pub fn count(&self) -> u32 {
        self.total
    }

    fn next_sample(&mut self, index: u32) -> Result<(i64, f64), String> {
        let ts = match index {
            0 => self.reader.read_bits(64)? as i64,
            1 => {
                let delta = unzigzag(self.reader.read_bits(64)?);
                self.prev_delta = delta;
                self.prev_ts.wrapping_add(delta)
            }
            _ => {
                let dod = if !self.reader.read_bit()? {
                    0
                } else if !self.reader.read_bit()? {
                    self.reader.read_bits(14)? as i64 - 8191
                } else if !self.reader.read_bit()? {
                    self.reader.read_bits(17)? as i64 - 65535
                } else if !self.reader.read_bit()? {
                    self.reader.read_bits(20)? as i64 - 524287
                } else {
                    unzigzag(self.reader.read_bits(64)?)
                };
                let delta = self.prev_delta.wrapping_add(dod);
                self.prev_delta = delta;
                self.prev_ts.wrapping_add(delta)
            }
        };
        let bits = if index == 0 {
            self.reader.read_bits(64)?
        } else if !self.reader.read_bit()? {
            self.prev_bits
        } else {
            if self.reader.read_bit()? {
                self.leading = self.reader.read_bits(5)? as u8;
                let significant = self.reader.read_bits(6)? as u8;
                let significant = if significant == 0 { 64 } else { significant };
                self.trailing = 64 - self.leading - significant;
            }
            let significant = 64 - self.leading - self.trailing;
            let meaningful = self.reader.read_bits(significant)?;
            self.prev_bits ^ (meaningful << self.trailing)
        };
        self.prev_ts = ts;
        self.prev_bits = bits;
        Ok((ts, f64::from_bits(bits)))
    }
}

/// A Gorilla decoder that owns its input chunk.
///
/// The ordinary [`Decoder`] borrows a chunk, which is the right shape for the
/// query path. A streaming metric compaction needs one decoder per input
/// series in a k-way merge, however, and cannot keep self-referential
/// `Vec<u8>`/decoder pairs. This owned variant keeps each current chunk alive
/// while yielding it one sample at a time; at most one chunk per input series
/// is resident instead of materialising an entire merged series.
pub struct OwnedDecoder {
    inner: Decoder<'static>,
}

impl OwnedDecoder {
    pub fn new(chunk: Vec<u8>) -> Result<Self, String> {
        if chunk.len() < 4 {
            return Err("gorilla chunk is shorter than its count header".to_string());
        }
        let count = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        Ok(Self {
            inner: Decoder {
                reader: BitReader::owned(chunk),
                remaining: count,
                total: count,
                prev_ts: 0,
                prev_delta: 0,
                prev_bits: 0,
                leading: 0,
                trailing: 0,
            },
        })
    }

    pub fn declared_count(&self) -> u32 {
        self.inner.total
    }
}

impl Iterator for OwnedDecoder {
    type Item = Result<(i64, f64), String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl Iterator for Decoder<'_> {
    type Item = Result<(i64, f64), String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let index = self.total - self.remaining;
        self.remaining -= 1;
        Some(self.next_sample(index).inspect_err(|_| {
            // A decode error poisons the rest of the stream; stop instead of
            // producing garbage samples after it.
            self.remaining = 0;
        }))
    }
}

/// Decode a whole chunk, failing on the first corrupt record.
pub fn decode_all(chunk: &[u8]) -> Result<Vec<(i64, f64)>, String> {
    Decoder::new(chunk)?.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(samples: &[(i64, f64)]) {
        let mut encoder = Encoder::new();
        for (ts, value) in samples {
            encoder.append(*ts, *value);
        }
        assert_eq!(encoder.count() as usize, samples.len());
        let chunk = encoder.close();
        let decoded = decode_all(&chunk).expect("decodes");
        assert_eq!(decoded.len(), samples.len());
        for ((ts, value), (decoded_ts, decoded_value)) in samples.iter().zip(&decoded) {
            assert_eq!(ts, decoded_ts);
            assert_eq!(
                value.to_bits(),
                decoded_value.to_bits(),
                "values round-trip bit-exactly, NaN payloads included"
            );
        }
    }

    #[test]
    fn a_regular_scrape_grid_round_trips_and_compresses() {
        let samples: Vec<(i64, f64)> = (0..1000)
            .map(|index| {
                (
                    1_772_000_000_000_000_000 + index * 10_000_000_000,
                    100.0 + (index % 7) as f64 * 0.25,
                )
            })
            .collect();
        let mut encoder = Encoder::new();
        for (ts, value) in &samples {
            encoder.append(*ts, *value);
        }
        let chunk = encoder.close();
        // A perfectly regular grid spends one bit per timestamp after the
        // second sample; the whole chunk must land far under raw 16B/sample.
        assert!(
            chunk.len() < samples.len() * 4,
            "1000 grid samples took {} bytes",
            chunk.len()
        );
        assert_eq!(decode_all(&chunk).expect("decodes").len(), 1000);
        round_trip(&samples);
    }

    #[test]
    fn pathological_delta_of_deltas_round_trip() {
        // Every bucket boundary, both signs, the 64-bit fallback, a backwards
        // timestamp, and duplicate timestamps.
        let samples: Vec<(i64, f64)> = vec![
            (0, 1.0),
            (1, 1.0),
            (2, 1.0),                       // dod 0
            (2 + 8193, 2.0),                // dod 8192, top of the 14-bit bucket
            (2 + 8193 * 2 + 1, 2.0),        // dod 1
            (2 + 8193 * 2 + 1 - 8191, 3.0), // negative delta: time goes back
            (i64::MAX / 2, 3.5),            // 64-bit fallback dod
            (i64::MAX / 2, 3.5),            // duplicate timestamp
            (i64::MAX / 2 + 65_536, 4.0),
            (i64::MAX / 2 + 65_536 - 524_287, -4.0),
        ];
        round_trip(&samples);
    }

    #[test]
    fn value_bit_patterns_round_trip_exactly() {
        let samples: Vec<(i64, f64)> = [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MIN_POSITIVE,
            f64::MAX,
            1.0,
            1.0000000000000002,
            -1e-300,
        ]
        .iter()
        .enumerate()
        .map(|(index, value)| (index as i64 * 1_000, *value))
        .collect();
        round_trip(&samples);
    }

    #[test]
    fn owned_decoder_round_trips_without_changing_the_borrowed_path() {
        let samples = vec![(10, 1.0), (10, 2.0), (12, 3.0), (20, 4.0)];
        let mut encoder = Encoder::new();
        for (ts, value) in &samples {
            encoder.append(*ts, *value);
        }
        let chunk = encoder.close();
        let decoder = OwnedDecoder::new(chunk).expect("owned decoder opens");
        let decoded: Vec<_> = decoder
            .collect::<Result<_, _>>()
            .expect("owned decoder reads");
        assert_eq!(decoded, samples);
    }

    #[test]
    fn constant_values_cost_one_bit_each() {
        let mut encoder = Encoder::new();
        for index in 0..10_000i64 {
            encoder.append(index * 15_000_000_000, 42.0);
        }
        let chunk = encoder.close();
        // 2 bits per sample (dod 0 + xor 0) after the header samples: well
        // under one byte each.
        assert!(
            chunk.len() < 10_000,
            "10k constant samples took {} bytes",
            chunk.len()
        );
        let decoded = decode_all(&chunk).expect("decodes");
        assert!(decoded.iter().all(|(_, value)| *value == 42.0));
    }

    #[test]
    fn empty_and_single_sample_chunks_are_valid() {
        let empty = Encoder::new().close();
        assert_eq!(decode_all(&empty).expect("decodes"), Vec::new());
        let mut one = Encoder::new();
        one.append(1_772_000_000_000_000_000, 0.5);
        round_trip(&[(1_772_000_000_000_000_000, 0.5)]);
        assert_eq!(
            decode_all(&one.close()).expect("decodes"),
            vec![(1_772_000_000_000_000_000, 0.5)]
        );
    }

    #[test]
    fn a_truncated_chunk_is_an_error_not_garbage() {
        let mut encoder = Encoder::new();
        for index in 0..100i64 {
            encoder.append(index * 1_000, index as f64 * 1.5);
        }
        let chunk = encoder.close();
        assert!(decode_all(&chunk[..chunk.len() - 4]).is_err());
        assert!(Decoder::new(&chunk[..2]).is_err());
    }

    #[test]
    fn byte_len_matches_the_closed_chunk() {
        let mut encoder = Encoder::new();
        for index in 0..57i64 {
            encoder.append(index * 3_000, (index % 5) as f64);
        }
        let advertised = encoder.byte_len();
        assert_eq!(advertised, encoder.close().len());
    }

    #[test]
    fn a_random_walk_round_trips() {
        let mut state = 0x5eed_2026u64;
        let mut draw = move || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };
        let mut ts = 1_700_000_000_000_000_000i64;
        let mut value = 500.0f64;
        let samples: Vec<(i64, f64)> = (0..5_000)
            .map(|_| {
                ts += (draw() % 30_000_000_000) as i64;
                value += (draw() % 1_000) as f64 / 10.0 - 50.0;
                (ts, value)
            })
            .collect();
        round_trip(&samples);
    }
}
