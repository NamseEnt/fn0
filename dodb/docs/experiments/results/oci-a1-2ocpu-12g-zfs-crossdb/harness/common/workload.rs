use std::collections::HashSet;
use std::time::Duration;

pub const BASE_SEED: u64 = 979_000_000;
pub const SEED_STRIDE: u64 = 1_009;
pub const WARMUP_SEED_MASK: u64 = 0xaaaa_0000;
pub const MEASURED_SEED_MASK: u64 = 0xbbbb_0000;
pub const WRITER_SEED_MASK: u64 = 0x1000_0000;
pub const LATENCY_RESERVOIR_LIMIT: usize = 16_384;
pub const TRACE_CHECKPOINT_INTERVAL: u64 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Distribution {
    Uniform,
    SameLeafHeavy,
    DifferentLeafHeavy,
    Hotspot,
}

impl Distribution {
    pub fn parse(value: &str) -> Self {
        match value {
            "uniform" => Self::Uniform,
            "same-leaf-heavy" | "compact-locality" => Self::SameLeafHeavy,
            "different-leaf-heavy" | "spread-locality" => Self::DifferentLeafHeavy,
            "hotspot" => Self::Hotspot,
            other => panic!("unknown key distribution {other:?}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::SameLeafHeavy => "same-leaf-heavy",
            Self::DifferentLeafHeavy => "different-leaf-heavy",
            Self::Hotspot => "hotspot",
        }
    }

    pub fn report_name(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::SameLeafHeavy => "compact-locality",
            Self::DifferentLeafHeavy => "spread-locality",
            Self::Hotspot => "hotspot",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkloadConfig {
    pub distribution: Distribution,
    pub working_set: usize,
    pub key_size: usize,
    pub value_size: usize,
    pub width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mutation {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRequest {
    pub primary_key: Vec<u8>,
    pub exclusive_after_sort_key: Vec<u8>,
    pub expected_rows: Vec<Mutation>,
}

#[derive(Clone, Debug)]
pub struct WorkloadGenerator {
    config: WorkloadConfig,
    worker_id: usize,
    operation: u64,
    state: u64,
}

impl WorkloadGenerator {
    pub fn new(config: WorkloadConfig, seed: u64, worker_id: usize) -> Self {
        Self {
            config,
            worker_id,
            operation: 0,
            state: seed ^ (worker_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
        }
    }

    pub fn next_transaction(&mut self) -> Vec<Mutation> {
        let mut keys = Vec::with_capacity(self.config.width);
        let mut seen = HashSet::with_capacity(self.config.width);
        for offset in 0..self.config.width {
            let index = self.next_index(offset);
            let key = self.key_for_index(index);
            if seen.insert(key.clone()) {
                keys.push(key);
            } else {
                let fallback = self.key_for_index(
                    self.config
                        .working_set
                        .saturating_add(self.worker_id.saturating_mul(1_000_000))
                        .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                        .saturating_add(offset),
                );
                assert!(
                    seen.insert(fallback.clone()),
                    "transaction key generator duplicated a key"
                );
                keys.push(fallback);
            }
        }
        self.operation = self.operation.wrapping_add(1);
        keys.into_iter()
            .enumerate()
            .map(|(offset, key)| Mutation {
                key,
                value: transaction_value_bytes(
                    self.config.value_size,
                    self.worker_id,
                    self.operation,
                    offset,
                ),
            })
            .collect()
    }

    pub fn next_query(&mut self, limit: usize) -> QueryRequest {
        let partition_count = self.config.working_set.min(128);
        let partition_index = self.random_bounded(partition_count);
        let partition_rows = (self.config.working_set - 1 - partition_index) / 128 + 1;
        let cursor_slots = partition_rows.saturating_sub(limit).max(1);
        let cursor_slot = self.random_bounded(cursor_slots);
        let cursor_index = partition_index + cursor_slot * 128;
        let cursor_key =
            key_for_index(self.config.distribution, self.config.key_size, cursor_index);
        let key_component_length = self.config.key_size / 2;
        let primary_key = cursor_key[..key_component_length].to_vec();
        let exclusive_after_sort_key = cursor_key[key_component_length..].to_vec();
        let expected_rows = (1..=limit)
            .map(|row_offset| {
                let row_index = cursor_index + row_offset * 128;
                Mutation {
                    key: key_for_index(self.config.distribution, self.config.key_size, row_index),
                    value: value_bytes(self.config.value_size, row_index as u64, 0),
                }
            })
            .collect();
        self.operation = self.operation.wrapping_add(1);
        QueryRequest {
            primary_key,
            exclusive_after_sort_key,
            expected_rows,
        }
    }

    fn next_index(&mut self, offset: usize) -> usize {
        let working_set = self.config.working_set;
        match self.config.distribution {
            Distribution::Uniform => self.random_bounded(working_set),
            Distribution::SameLeafHeavy => {
                let span = working_set.min(64).max(self.config.width);
                ((self.operation as usize)
                    .saturating_mul(self.config.width)
                    .saturating_add(offset))
                    % span.min(working_set)
            }
            Distribution::DifferentLeafHeavy => {
                (self
                    .worker_id
                    .saturating_mul(1_009)
                    .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                    .saturating_add(offset))
                    % working_set
            }
            Distribution::Hotspot => {
                let hot_set = (working_set / 100).max(1);
                if self.random_bounded(100) < 80 {
                    self.random_bounded(hot_set)
                } else {
                    self.random_bounded(working_set)
                }
            }
        }
    }

    fn random_bounded(&mut self, bound: usize) -> usize {
        assert!(bound > 0);
        self.state = splitmix64(self.state);
        (self.state as usize) % bound
    }

    pub fn key_for_index(&self, index: usize) -> Vec<u8> {
        key_for_index(self.config.distribution, self.config.key_size, index)
    }
}

pub fn key_for_index(distribution: Distribution, key_size: usize, index: usize) -> Vec<u8> {
    let (pk_len, sk_len) = key_component_lengths(key_size);
    let (pk_tag, pk_value, sk_tag) = match distribution {
        Distribution::SameLeafHeavy => (0x11, 0, 0x21),
        Distribution::DifferentLeafHeavy => (0x31, index as u64, 0x41),
        Distribution::Uniform => (0x51, (index % 128) as u64, 0x61),
        Distribution::Hotspot => (0x51, (index % 128) as u64, 0x61),
    };
    let mut key = component_bytes(pk_tag, pk_value, pk_len);
    key.extend_from_slice(&component_bytes(sk_tag, index as u64, sk_len));
    key
}

pub fn seed_rows(config: &WorkloadConfig) -> impl Iterator<Item = Mutation> + '_ {
    (0..config.working_set).map(move |index| Mutation {
        key: key_for_index(config.distribution, config.key_size, index),
        value: value_bytes(config.value_size, index as u64, 0),
    })
}

pub fn invocation_seed(scenario_index: u64, repetition_index: u64) -> u64 {
    BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
}

pub fn writer_phase_seed(invocation_seed: u64, warmup: bool) -> u64 {
    let phase_mask = if warmup {
        WARMUP_SEED_MASK
    } else {
        MEASURED_SEED_MASK
    };
    invocation_seed ^ phase_mask ^ WRITER_SEED_MASK
}

pub fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub fn key_component_lengths(key_size: usize) -> (usize, usize) {
    let pk_len = (key_size / 2).max(1);
    let sk_len = key_size.saturating_sub(pk_len).max(1);
    (pk_len, sk_len)
}

pub fn component_bytes(tag: u8, value: u64, length: usize) -> Vec<u8> {
    let mut bytes = vec![tag; length];
    let encoded = value.to_be_bytes();
    let copy_len = encoded.len().min(length);
    bytes[length - copy_len..].copy_from_slice(&encoded[encoded.len() - copy_len..]);
    bytes
}

pub fn value_bytes(length: usize, operation: u64, offset: usize) -> Vec<u8> {
    let mut value = Vec::with_capacity(length);
    let mut block_index = 0u64;
    while value.len() < length {
        let block = splitmix64(
            operation
                .rotate_left(19)
                .wrapping_add(offset as u64)
                .wrapping_add(block_index.wrapping_mul(0x9e37_79b9_7f4a_7c15)),
        )
        .to_be_bytes();
        let remaining = length - value.len();
        value.extend_from_slice(&block[..remaining.min(block.len())]);
        block_index = block_index.wrapping_add(1);
    }
    value
}

pub fn transaction_value_bytes(
    length: usize,
    worker_id: usize,
    operation_id: u64,
    mutation_offset: usize,
) -> Vec<u8> {
    assert!(length >= 28);
    let worker_identity = worker_id as u64;
    let mutation_identity = mutation_offset as u64;
    let mut value = value_bytes(
        length,
        operation_id ^ worker_identity.rotate_left(13),
        mutation_offset ^ worker_id,
    );
    value[..4].copy_from_slice(b"DDBW");
    value[4..12].copy_from_slice(&worker_identity.to_be_bytes());
    value[12..20].copy_from_slice(&operation_id.to_be_bytes());
    value[20..28].copy_from_slice(&mutation_identity.to_be_bytes());
    value
}

pub fn transaction_value_identity(bytes: &[u8]) -> Option<(u64, u64, u64)> {
    if bytes.len() < 28 || &bytes[..4] != b"DDBW" {
        return None;
    }
    Some((
        u64::from_be_bytes(bytes[4..12].try_into().ok()?),
        u64::from_be_bytes(bytes[12..20].try_into().ok()?),
        u64::from_be_bytes(bytes[20..28].try_into().ok()?),
    ))
}

#[derive(Clone, Debug)]
pub struct TraceHash {
    pub transactions: u64,
    pub state: u64,
    pub checkpoints: Vec<(u64, u64)>,
}

impl TraceHash {
    pub fn new() -> Self {
        Self {
            transactions: 0,
            state: 0xcbf2_9ce4_8422_2325,
            checkpoints: Vec::new(),
        }
    }

    fn absorb(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    pub fn push_transaction<'a>(
        &mut self,
        mutations: impl IntoIterator<Item = (&'a [u8], &'a [u8])>,
    ) {
        self.transactions += 1;
        let mut width = 0u32;
        for (key, value) in mutations {
            width += 1;
            self.absorb(&(key.len() as u32).to_be_bytes());
            self.absorb(key);
            self.absorb(&(value.len() as u32).to_be_bytes());
            self.absorb(value);
        }
        self.absorb(&width.to_be_bytes());
        self.record_checkpoint();
    }

    pub fn push_get(&mut self, key: &[u8]) {
        self.transactions += 1;
        self.absorb(b"get");
        self.absorb(&(key.len() as u32).to_be_bytes());
        self.absorb(key);
        self.record_checkpoint();
    }

    pub fn push_query(&mut self, request: &QueryRequest, limit: usize) {
        self.transactions += 1;
        self.absorb(b"query");
        self.absorb(&(request.primary_key.len() as u32).to_be_bytes());
        self.absorb(&request.primary_key);
        self.absorb(&(request.exclusive_after_sort_key.len() as u32).to_be_bytes());
        self.absorb(&request.exclusive_after_sort_key);
        self.absorb(&(limit as u64).to_be_bytes());
        self.record_checkpoint();
    }

    fn record_checkpoint(&mut self) {
        if self.transactions.is_multiple_of(TRACE_CHECKPOINT_INTERVAL) {
            self.checkpoints.push((self.transactions, self.state));
        }
    }
}

impl Default for TraceHash {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LatencySamples {
    pub values: Vec<Duration>,
    seen: u64,
    state: u64,
}

impl LatencySamples {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            values: Vec::new(),
            seen: 0,
            state: seed,
        }
    }

    pub fn push(&mut self, value: Duration) {
        self.seen = self.seen.saturating_add(1);
        if self.values.len() < LATENCY_RESERVOIR_LIMIT {
            self.values.push(value);
            return;
        }
        self.state = splitmix64(self.state);
        let index = (self.state % self.seen) as usize;
        if index < LATENCY_RESERVOIR_LIMIT {
            self.values[index] = value;
        }
    }

    pub fn merge(&mut self, other: Self) {
        for value in other.values {
            self.push(value);
        }
    }

    pub fn percentile_us(&self, fraction: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut values = self.values.clone();
        values.sort_unstable();
        let index = ((values.len().saturating_sub(1)) as f64 * fraction).round() as usize;
        values[index].as_secs_f64() * 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Distribution, LATENCY_RESERVOIR_LIMIT, LatencySamples, WorkloadConfig, WorkloadGenerator,
        transaction_value_bytes, transaction_value_identity,
    };
    use std::collections::HashSet;
    use std::time::Duration;

    #[test]
    fn query_requests_cover_partitions_cursors_and_exact_seed_rows() {
        let config = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            width: 1,
        };
        let mut generator = WorkloadGenerator::new(config, 0x1234_5678, 7);
        let mut partitions = HashSet::new();
        let mut cursors = HashSet::new();
        for _ in 0..256 {
            let request = generator.next_query(16);
            partitions.insert(request.primary_key.clone());
            cursors.insert(request.exclusive_after_sort_key.clone());
            assert_eq!(request.expected_rows.len(), 16);
            let mut previous_key = [
                request.primary_key.as_slice(),
                request.exclusive_after_sort_key.as_slice(),
            ]
            .concat();
            for row in request.expected_rows {
                assert_eq!(&row.key[..8], request.primary_key.as_slice());
                assert!(row.key.as_slice() > previous_key.as_slice());
                assert_eq!(row.value.len(), 512);
                previous_key = row.key;
            }
        }
        assert!(partitions.len() > 1);
        assert!(cursors.len() > 1);
    }

    #[test]
    fn latency_reservoir_stays_within_the_shared_sample_budget() {
        let mut samples = LatencySamples::with_seed(0xfeed_beef);
        for sample_index in 0..(LATENCY_RESERVOIR_LIMIT * 4) {
            samples.push(Duration::from_nanos(sample_index as u64));
        }
        assert_eq!(samples.values.len(), LATENCY_RESERVOIR_LIMIT);
        assert_eq!(samples.seen, (LATENCY_RESERVOIR_LIMIT * 4) as u64);
    }

    #[test]
    fn transaction_values_change_across_the_full_payload() {
        let first = transaction_value_bytes(512, 2, 1, 0);
        let next = transaction_value_bytes(512, 2, 2, 0);
        assert_eq!(first.len(), 512);
        assert_eq!(next.len(), 512);
        assert_ne!(first, next);
        assert_eq!(transaction_value_identity(&first), Some((2, 1, 0)));
    }
}
