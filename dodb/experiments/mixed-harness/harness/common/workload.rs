use std::collections::HashSet;
use std::time::Duration;

pub const BASE_SEED: u64 = 979_000_000;
pub const SEED_STRIDE: u64 = 1_009;
pub const WARMUP_SEED_MASK: u64 = 0xaaaa_0000;
pub const MEASURED_SEED_MASK: u64 = 0xbbbb_0000;
pub const WRITER_SEED_MASK: u64 = 0x1000_0000;
pub const LATENCY_RESERVOIR_LIMIT: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Distribution {
    Uniform,
    SameLeafHeavy,
    DifferentLeafHeavy,
    Hotspot,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MixedValueMode {
    #[default]
    Constant,
    Changing,
}

impl MixedValueMode {
    pub fn parse(value: &str) -> Self {
        match value {
            "constant" => Self::Constant,
            "changing" => Self::Changing,
            other => panic!("unknown mixed value mode {other:?}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Changing => "changing",
        }
    }

    pub fn generator_name(self) -> &'static str {
        match self {
            Self::Constant => "legacy_constant_v1",
            Self::Changing => "seeded_nonrepeating_v1",
        }
    }
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

#[derive(Clone, Debug)]
pub struct WorkloadGenerator {
    config: WorkloadConfig,
    worker_id: usize,
    operation: u64,
    state: u64,
    mixed_value_context: Option<MixedValueContext>,
    mixed_value_mode: MixedValueMode,
}

#[derive(Clone, Copy, Debug)]
struct MixedValueContext {
    phase_seed: u64,
    operation_seed: u64,
    operation_index: u64,
}

impl WorkloadGenerator {
    pub fn new(config: WorkloadConfig, seed: u64, worker_id: usize) -> Self {
        Self {
            config,
            worker_id,
            operation: 0,
            state: seed ^ (worker_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            mixed_value_context: None,
            mixed_value_mode: MixedValueMode::Constant,
        }
    }

    pub fn new_mixed(
        config: WorkloadConfig,
        operation_seed: u64,
        phase_seed: u64,
        operation_index: u64,
        value_mode: MixedValueMode,
    ) -> Self {
        let mut generator = Self::new(config, operation_seed, 0);
        generator.mixed_value_mode = value_mode;
        if value_mode == MixedValueMode::Changing {
            generator.mixed_value_context = Some(MixedValueContext {
                phase_seed,
                operation_seed,
                operation_index,
            });
        }
        generator
    }

    pub fn next_transaction(&mut self) -> Vec<Mutation> {
        self.next_keys()
            .into_iter()
            .enumerate()
            .map(|(offset, key)| Mutation {
                key,
                value: match (self.mixed_value_mode, self.mixed_value_context) {
                    (MixedValueMode::Changing, Some(context)) => mixed_value_bytes(
                        self.config.value_size,
                        context.phase_seed,
                        context.operation_seed,
                        context.operation_index,
                        offset,
                    ),
                    _ => value_bytes(self.config.value_size, self.operation, offset),
                },
            })
            .collect()
    }

    pub fn next_read_key(&mut self) -> Vec<u8> {
        self.next_keys()
            .into_iter()
            .next()
            .expect("transaction width must be positive")
    }

    fn next_keys(&mut self) -> Vec<Vec<u8>> {
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
        keys
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

pub fn mixed_operation_is_read(operation_index: u64, read_percent: u8) -> bool {
    operation_index % 100 < u64::from(read_percent)
}

pub fn mixed_operation_seed(phase_seed: u64, operation_index: u64) -> u64 {
    splitmix64(phase_seed ^ operation_index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
}

pub fn mixed_value_bytes(
    length: usize,
    phase_seed: u64,
    operation_seed: u64,
    operation_index: u64,
    mutation_index: usize,
) -> Vec<u8> {
    let mut bytes = vec![0; length];
    let nonce = operation_index.to_be_bytes();
    let nonce_length = length.min(nonce.len());
    bytes[..nonce_length].copy_from_slice(&nonce[nonce.len() - nonce_length..]);

    let mut state = phase_seed
        ^ operation_seed.rotate_left(17)
        ^ operation_index.wrapping_mul(0xd6e8_feb8_6659_fd93)
        ^ (mutation_index as u64).wrapping_mul(0xa076_1d64_78bd_642f);
    let mut position = nonce_length;
    while position < length {
        state = splitmix64(state);
        for byte in state.to_be_bytes() {
            if position >= length {
                break;
            }
            bytes[position] = byte;
            position += 1;
        }
    }
    bytes
}

pub fn mixed_trace_prefix_hash(
    config: WorkloadConfig,
    phase_seed: u64,
    read_percent: u8,
    value_mode: MixedValueMode,
    prefix_operations: u64,
) -> u64 {
    let mut state = 0xcbf2_9ce4_8422_2325;
    for operation_index in 0..prefix_operations {
        let is_read = mixed_operation_is_read(operation_index, read_percent);
        let operation_seed = mixed_operation_seed(phase_seed, operation_index);
        let mut generator = WorkloadGenerator::new_mixed(
            config.clone(),
            operation_seed,
            phase_seed,
            operation_index,
            value_mode,
        );
        let mutations = generator.next_transaction();
        absorb_trace(&mut state, &operation_index.to_be_bytes());
        absorb_trace(&mut state, &[u8::from(is_read)]);
        if is_read {
            absorb_trace(&mut state, &(mutations[0].key.len() as u32).to_be_bytes());
            absorb_trace(&mut state, &mutations[0].key);
        } else {
            absorb_trace(&mut state, &(mutations.len() as u32).to_be_bytes());
            for mutation in mutations {
                absorb_trace(&mut state, &(mutation.key.len() as u32).to_be_bytes());
                absorb_trace(&mut state, &mutation.key);
                absorb_trace(&mut state, &(mutation.value.len() as u32).to_be_bytes());
                absorb_trace(&mut state, &mutation.value);
            }
        }
    }
    state
}

fn absorb_trace(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *state ^= u64::from(*byte);
        *state = state.wrapping_mul(0x0000_0100_0000_01b3);
    }
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

#[cfg(test)]
mod tests {
    use super::{
        Distribution, LATENCY_RESERVOIR_LIMIT, LatencySamples, MixedValueMode, WorkloadConfig,
        WorkloadGenerator, mixed_operation_is_read, mixed_operation_seed, mixed_trace_prefix_hash,
    };
    use std::time::Duration;

    #[test]
    fn latency_samples_keep_a_bounded_deterministic_reservoir() {
        let mut samples = LatencySamples::with_seed(0x1234_5678);
        for sample_index in 0..(LATENCY_RESERVOIR_LIMIT * 4) {
            samples.push(Duration::from_nanos(sample_index as u64));
        }

        assert_eq!(samples.values.len(), LATENCY_RESERVOIR_LIMIT);
        assert_eq!(samples.seen, (LATENCY_RESERVOIR_LIMIT * 4) as u64);
        assert!(samples.percentile_us(0.99).is_finite());
    }

    #[test]
    fn mixed_schedule_preserves_each_ratio_per_hundred_operations() {
        for read_percent in [95, 50, 20] {
            let reads = (0..10_000)
                .filter(|operation_index| mixed_operation_is_read(*operation_index, read_percent))
                .count();
            assert_eq!(reads, 100 * usize::from(read_percent));
        }
    }

    #[test]
    fn mixed_trace_prefix_is_seeded_and_width_sensitive() {
        let config = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            width: 4,
        };
        let trace_hash = mixed_trace_prefix_hash(
            config.clone(),
            0x1234_5678_9abc_def0,
            95,
            MixedValueMode::Constant,
            1_000,
        );
        assert_eq!(trace_hash, 0x3cca_5e07_ae2e_0ae5);
        let changing_trace_hash = mixed_trace_prefix_hash(
            config.clone(),
            0x1234_5678_9abc_def0,
            95,
            MixedValueMode::Changing,
            1_000,
        );
        assert_eq!(changing_trace_hash, 0xd6ea_8c55_50fc_11e1);
        assert_eq!(
            trace_hash,
            mixed_trace_prefix_hash(
                config.clone(),
                0x1234_5678_9abc_def0,
                95,
                MixedValueMode::Constant,
                1_000,
            )
        );
        assert_ne!(
            trace_hash,
            mixed_trace_prefix_hash(
                config,
                0x1234_5678_9abc_def1,
                95,
                MixedValueMode::Constant,
                1_000,
            )
        );
    }

    #[test]
    fn changing_mixed_values_are_seeded_and_distinct_per_operation() {
        let config = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            width: 1,
        };
        let phase_seed = 0x1234_5678_9abc_def0;
        let value_for = |operation_index| {
            let operation_seed = mixed_operation_seed(phase_seed, operation_index);
            let mut generator = WorkloadGenerator::new_mixed(
                config.clone(),
                operation_seed,
                phase_seed,
                operation_index,
                MixedValueMode::Changing,
            );
            generator.next_transaction().remove(0).value
        };
        let first_value = value_for(0);
        let second_value = value_for(1);
        assert_eq!(first_value.len(), 512);
        assert_eq!(first_value, value_for(0));
        assert_ne!(first_value, second_value);
        assert!(first_value[8..].iter().any(|byte| *byte != first_value[8]));
        assert_eq!(first_value[..8], 0u64.to_be_bytes());
        assert_eq!(second_value[..8], 1u64.to_be_bytes());
    }

    #[test]
    fn constant_mixed_values_preserve_the_previous_request_bytes() {
        let config = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            width: 1,
        };
        let first_operation = 3;
        let second_operation = 991;
        for operation_index in [first_operation, second_operation] {
            let phase_seed = 0x1234_5678_9abc_def0;
            let operation_seed = mixed_operation_seed(phase_seed, operation_index);
            let mut generator = WorkloadGenerator::new_mixed(
                config.clone(),
                operation_seed,
                phase_seed,
                operation_index,
                MixedValueMode::Constant,
            );
            assert_eq!(generator.next_transaction()[0].value, vec![1; 512]);
        }
    }

    #[test]
    fn read_key_generation_preserves_keys_state_and_following_write_values() {
        for distribution in [
            Distribution::Uniform,
            Distribution::SameLeafHeavy,
            Distribution::DifferentLeafHeavy,
            Distribution::Hotspot,
        ] {
            for width in [1, 4] {
                for value_mode in [MixedValueMode::Constant, MixedValueMode::Changing] {
                    let config = WorkloadConfig {
                        distribution,
                        working_set: 1_024,
                        key_size: 16,
                        value_size: 512,
                        width,
                    };
                    let phase_seed = 0x1234_5678_9abc_def0;
                    let operation_seed = mixed_operation_seed(phase_seed, 17);
                    let mut reference_generator = WorkloadGenerator::new_mixed(
                        config.clone(),
                        operation_seed,
                        phase_seed,
                        17,
                        value_mode,
                    );
                    let mut read_key_generator = WorkloadGenerator::new_mixed(
                        config,
                        operation_seed,
                        phase_seed,
                        17,
                        value_mode,
                    );
                    for operation_index in 0..8 {
                        let reference_read = reference_generator.next_transaction();
                        let read_key = read_key_generator.next_read_key();
                        assert_eq!(
                            read_key, reference_read[0].key,
                            "read key differed for {distribution:?}, width {width}, mode {value_mode:?}, operation {operation_index}"
                        );
                        assert_eq!(
                            reference_generator.next_transaction(),
                            read_key_generator.next_transaction(),
                            "following write differed for {distribution:?}, width {width}, mode {value_mode:?}, operation {operation_index}"
                        );
                    }
                }
            }
        }
    }
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
