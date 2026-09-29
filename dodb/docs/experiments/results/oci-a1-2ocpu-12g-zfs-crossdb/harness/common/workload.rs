use std::collections::HashSet;
use std::time::Duration;

pub const BASE_SEED: u64 = 979_000_000;
pub const SEED_STRIDE: u64 = 1_009;
pub const WARMUP_SEED_MASK: u64 = 0xaaaa_0000;
pub const MEASURED_SEED_MASK: u64 = 0xbbbb_0000;
pub const WRITER_SEED_MASK: u64 = 0x1000_0000;
pub const LATENCY_RESERVOIR_LIMIT: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Distribution {
    Uniform,
    SameLeafHeavy,
    DifferentLeafHeavy,
}

impl Distribution {
    pub fn parse(value: &str) -> Self {
        match value {
            "uniform" => Self::Uniform,
            "same-leaf-heavy" | "compact-locality" => Self::SameLeafHeavy,
            "different-leaf-heavy" | "spread-locality" => Self::DifferentLeafHeavy,
            other => panic!("unknown key distribution {other:?}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::SameLeafHeavy => "same-leaf-heavy",
            Self::DifferentLeafHeavy => "different-leaf-heavy",
        }
    }

    pub fn report_name(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::SameLeafHeavy => "compact-locality",
            Self::DifferentLeafHeavy => "spread-locality",
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
                value: value_bytes(self.config.value_size, self.operation, offset),
            })
            .collect()
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
    let byte = (operation.wrapping_add(offset as u64) & 0xff) as u8;
    vec![byte; length]
}

#[derive(Clone, Copy, Debug)]
pub struct TraceHash {
    pub transactions: u64,
    pub state: u64,
}

impl TraceHash {
    pub fn new() -> Self {
        Self {
            transactions: 0,
            state: 0xcbf2_9ce4_8422_2325,
        }
    }

    fn absorb(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    pub fn push_transaction<'a>(&mut self, mutations: impl IntoIterator<Item = (&'a [u8], &'a [u8])>) {
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
