//! Deterministic workload, reference-model, and reporting primitives for the
//! `dodb-soak` executable.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::fmt;
use std::time::Duration;

use dodb_core::{DocumentKey, PrimaryKey, Revision, RevisionState, TenantId};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// The operation mix is deliberately explicit so a run can be reproduced
/// without relying on process-global or thread-local entropy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OperationKind {
    Get,
    Put,
    Delete,
    Query,
    Scan,
    ConcurrentGet,
    Transact,
}

impl OperationKind {
    pub const ALL: [Self; 7] = [
        Self::Get,
        Self::Put,
        Self::Delete,
        Self::Query,
        Self::Scan,
        Self::ConcurrentGet,
        Self::Transact,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Query => "query",
            Self::Scan => "scan",
            Self::ConcurrentGet => "concurrent_get",
            Self::Transact => "transact",
        }
    }

    pub const fn counter_index(self) -> usize {
        match self {
            Self::Get => 0,
            Self::Put => 1,
            Self::Delete => 2,
            Self::Query => 3,
            Self::Scan => 4,
            Self::ConcurrentGet => 5,
            Self::Transact => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlannedMutationKind {
    Put,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedMutation {
    pub key: DocumentKey,
    pub kind: PlannedMutationKind,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedOperation {
    Get {
        tenant: TenantId,
        key: DocumentKey,
    },
    Put {
        tenant: TenantId,
        key: DocumentKey,
        value: Vec<u8>,
    },
    Delete {
        tenant: TenantId,
        key: DocumentKey,
    },
    Query {
        tenant: TenantId,
        pk: PrimaryKey,
        exclusive_after_sk: Option<dodb_core::SortKey>,
        limit: usize,
    },
    Scan {
        tenant: TenantId,
        exclusive_after_key: Option<DocumentKey>,
        limit: usize,
    },
    ConcurrentGet {
        tenant: TenantId,
        keys: Vec<DocumentKey>,
    },
    Transact {
        tenant: TenantId,
        mutations: Vec<PlannedMutation>,
    },
}

impl GeneratedOperation {
    pub const fn kind(&self) -> OperationKind {
        match self {
            Self::Get { .. } => OperationKind::Get,
            Self::Put { .. } => OperationKind::Put,
            Self::Delete { .. } => OperationKind::Delete,
            Self::Query { .. } => OperationKind::Query,
            Self::Scan { .. } => OperationKind::Scan,
            Self::ConcurrentGet { .. } => OperationKind::ConcurrentGet,
            Self::Transact { .. } => OperationKind::Transact,
        }
    }

    pub fn tenant(&self) -> TenantId {
        match self {
            Self::Get { tenant, .. }
            | Self::Put { tenant, .. }
            | Self::Delete { tenant, .. }
            | Self::Query { tenant, .. }
            | Self::Scan { tenant, .. }
            | Self::ConcurrentGet { tenant, .. }
            | Self::Transact { tenant, .. } => *tenant,
        }
    }

    pub fn primary_key(&self) -> Option<&PrimaryKey> {
        match self {
            Self::Query { pk, .. } => Some(pk),
            _ => None,
        }
    }

    pub fn keys(&self) -> Vec<&DocumentKey> {
        match self {
            Self::Get { key, .. } | Self::Put { key, .. } | Self::Delete { key, .. } => vec![key],
            Self::Query { .. } => Vec::new(),
            Self::Scan {
                exclusive_after_key,
                ..
            } => exclusive_after_key.iter().collect(),
            Self::ConcurrentGet { keys, .. } => keys.iter().collect(),
            Self::Transact { mutations, .. } => {
                mutations.iter().map(|mutation| &mutation.key).collect()
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkloadConfig {
    pub tenant_count: u64,
    pub hot_key_count: u32,
    pub wide_key_count: u32,
    pub bounded_keyspace: bool,
    /// Growth phases fill this many ordinary keys per tenant before reusing
    /// the same finite keyspace for churn. Bounded phases use wide_key_count.
    pub growth_target_key_count: u32,
    /// Number of generated operations spent filling the growth keyspace.
    pub growth_fill_operations: u64,
    /// Finite key count for the explicit put/delete transition probes.
    pub transition_key_count: u32,
    /// Finite key count for reused large-value probe keys.
    pub large_probe_key_count: u32,
    pub hot_percent: u32,
    pub operation_weights: [u32; 7],
    pub max_query_limit: usize,
    pub max_scan_limit: usize,
    pub max_transaction_mutations: usize,
}

impl Default for WorkloadConfig {
    fn default() -> Self {
        Self {
            tenant_count: 16,
            hot_key_count: 8,
            wide_key_count: 512,
            bounded_keyspace: false,
            growth_target_key_count: 1_024,
            growth_fill_operations: 16_384,
            transition_key_count: 16,
            large_probe_key_count: 2,
            hot_percent: 80,
            operation_weights: [35, 20, 10, 10, 5, 10, 10],
            max_query_limit: 32,
            max_scan_limit: 64,
            max_transaction_mutations: 3,
        }
    }
}

impl WorkloadConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.tenant_count == 0
            || self.hot_key_count == 0
            || self.wide_key_count == 0
            || self.growth_target_key_count == 0
            || self.growth_fill_operations == 0
            || self.transition_key_count == 0
            || self.large_probe_key_count == 0
            || self.max_query_limit == 0
            || self.max_scan_limit == 0
            || self.max_transaction_mutations == 0
        {
            return Err("tenant, key, limit, and transaction counts must be nonzero".to_owned());
        }
        if self.hot_percent > 100 {
            return Err("hot-percent must be between 0 and 100".to_owned());
        }
        if self.operation_weights.iter().sum::<u32>() == 0 {
            return Err("at least one operation weight must be nonzero".to_owned());
        }
        if !self.bounded_keyspace
            && self.growth_fill_operations
                < self
                    .tenant_count
                    .checked_mul(u64::from(self.growth_target_key_count))
                    .ok_or_else(|| "growth key budget overflows u64".to_owned())?
        {
            return Err(
                "growth fill operations must cover every tenant's target key count".to_owned(),
            );
        }
        Ok(())
    }

    pub fn maximum_generated_key_count(&self) -> usize {
        let ordinary = if self.bounded_keyspace {
            self.wide_key_count
        } else {
            self.growth_target_key_count
        } as usize;
        (self.tenant_count as usize).saturating_mul(
            self.hot_key_count as usize
                + ordinary
                + self.transition_key_count as usize
                + self.large_probe_key_count as usize,
        )
    }
}

#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 7;
        value ^= value >> 9;
        value ^= value << 8;
        self.state = value;
        value
    }

    fn below(&mut self, upper: u64) -> u64 {
        if upper == 0 {
            0
        } else {
            self.next_u64() % upper
        }
    }
}

pub struct OperationGenerator {
    rng: DeterministicRng,
    config: WorkloadConfig,
}

impl OperationGenerator {
    pub fn new(seed: u64, config: WorkloadConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            rng: DeterministicRng::new(seed),
            config,
        })
    }

    pub fn next(&mut self, operation_index: u64) -> GeneratedOperation {
        if let Some(operation) = self.growth_seed_operation(operation_index) {
            return operation;
        }
        if let Some(operation) = self.large_value_probe_operation(operation_index) {
            return operation;
        }
        if let Some(operation) = self.transition_operation(operation_index) {
            return operation;
        }
        let kind = self.choose_kind();
        let tenant = TenantId::new(self.rng.below(self.config.tenant_count) + 1);
        match kind {
            OperationKind::Get => GeneratedOperation::Get {
                tenant,
                key: self.key(tenant, operation_index),
            },
            OperationKind::Put => {
                let key = self.key(tenant, operation_index);
                let value = self.value_for_key(operation_index, &key);
                GeneratedOperation::Put { tenant, key, value }
            }
            OperationKind::Delete => GeneratedOperation::Delete {
                tenant,
                key: self.wide_key(tenant, operation_index),
            },
            OperationKind::Query => {
                let pk = self.primary_key(tenant);
                let exclusive_after_sk = (self.rng.below(4) == 0).then(|| {
                    dodb_core::SortKey::new(format!(
                        "hot-sk-{}",
                        self.rng.below(u64::from(self.config.hot_key_count))
                    ))
                });
                GeneratedOperation::Query {
                    tenant,
                    pk,
                    exclusive_after_sk,
                    limit: self.limit(self.config.max_query_limit),
                }
            }
            OperationKind::Scan => GeneratedOperation::Scan {
                tenant,
                exclusive_after_key: (self.rng.below(5) == 0)
                    .then(|| self.key(tenant, operation_index)),
                limit: self.limit(self.config.max_scan_limit),
            },
            OperationKind::ConcurrentGet => {
                let key_count = 1 + self.rng.below(4) as usize;
                GeneratedOperation::ConcurrentGet {
                    tenant,
                    keys: (0..key_count)
                        .map(|_| self.key(tenant, operation_index))
                        .collect(),
                }
            }
            OperationKind::Transact => {
                let mutation_count =
                    1 + self.rng.below(self.config.max_transaction_mutations as u64) as usize;
                let mut mutations = Vec::with_capacity(mutation_count);
                while mutations.len() < mutation_count {
                    let key = self.key(tenant, operation_index);
                    if mutations
                        .iter()
                        .any(|mutation: &PlannedMutation| mutation.key == key)
                    {
                        continue;
                    }
                    let kind = if self.rng.below(4) == 0 {
                        PlannedMutationKind::Delete
                    } else {
                        PlannedMutationKind::Put
                    };
                    let value = self.value_for_key(operation_index, &key);
                    mutations.push(PlannedMutation { key, kind, value });
                }
                GeneratedOperation::Transact { tenant, mutations }
            }
        }
    }

    fn transition_operation(&mut self, operation_index: u64) -> Option<GeneratedOperation> {
        let step = operation_index % 64;
        if step >= 4 {
            return None;
        }
        let tenant = TenantId::new((operation_index % self.config.tenant_count) + 1);
        let key = DocumentKey::new(
            format!(
                "tenant-{}-hot-transition-pk-{}",
                tenant.get(),
                (operation_index / 64) % u64::from(self.config.transition_key_count)
            ),
            "transition-sk",
        );
        let value = self.value_for_key(operation_index, &key);
        Some(match step {
            0 | 2 => GeneratedOperation::Put { tenant, key, value },
            _ => GeneratedOperation::Delete { tenant, key },
        })
    }

    fn growth_seed_operation(&mut self, operation_index: u64) -> Option<GeneratedOperation> {
        if self.config.bounded_keyspace || operation_index >= self.config.growth_fill_operations {
            return None;
        }
        let tenant = TenantId::new((operation_index % self.config.tenant_count) + 1);
        let index = operation_index / self.config.tenant_count;
        let key = self.wide_key_at(tenant, index);
        let value_size = if index.is_multiple_of(8) {
            2 * 1024
        } else {
            128
        };
        Some(GeneratedOperation::Put {
            tenant,
            key,
            value: patterned_value(operation_index, value_size),
        })
    }

    fn choose_kind(&mut self) -> OperationKind {
        let total = self.config.operation_weights.iter().sum::<u32>();
        let mut choice = self.rng.below(u64::from(total)) as u32;
        for (index, weight) in self.config.operation_weights.iter().enumerate() {
            if choice < *weight {
                return OperationKind::ALL[index];
            }
            choice -= *weight;
        }
        OperationKind::Transact
    }

    fn key(&mut self, tenant: TenantId, operation_index: u64) -> DocumentKey {
        if self.rng.below(100) < u64::from(self.config.hot_percent) {
            let index = self.rng.below(u64::from(self.config.hot_key_count)) as u32;
            self.hot_key(tenant, index)
        } else {
            self.wide_key(tenant, operation_index)
        }
    }

    fn wide_key(&mut self, tenant: TenantId, operation_index: u64) -> DocumentKey {
        let key_count = if self.config.bounded_keyspace {
            self.config.wide_key_count
        } else {
            self.config.growth_target_key_count
        };
        let _ = operation_index;
        let index = self.rng.below(u64::from(key_count));
        self.wide_key_at(tenant, index)
    }

    fn wide_key_at(&self, tenant: TenantId, index: u64) -> DocumentKey {
        let key_count = if self.config.bounded_keyspace {
            self.config.wide_key_count
        } else {
            self.config.growth_target_key_count
        };
        let index = index % u64::from(key_count);
        DocumentKey::new(
            format!("tenant-{}-wide-pk-{}", tenant.get(), index % 64),
            format!("wide-sk-{index}"),
        )
    }

    fn hot_key(&self, tenant: TenantId, index: u32) -> DocumentKey {
        DocumentKey::new(
            format!("tenant-{}-hot-pk-{}", tenant.get(), index % 8),
            format!("hot-sk-{index}"),
        )
    }

    fn large_probe_key(&self, tenant: TenantId, index: u32) -> DocumentKey {
        DocumentKey::new(
            format!("tenant-{}-large-probe-pk-{}", tenant.get(), index),
            format!("large-probe-sk-{index}"),
        )
    }

    fn primary_key(&mut self, tenant: TenantId) -> PrimaryKey {
        if self.rng.below(100) < u64::from(self.config.hot_percent) {
            PrimaryKey::new(format!(
                "tenant-{}-hot-pk-{}",
                tenant.get(),
                self.rng.below(8)
            ))
        } else {
            PrimaryKey::new(format!(
                "tenant-{}-wide-pk-{}",
                tenant.get(),
                self.rng.below(64)
            ))
        }
    }

    fn limit(&mut self, maximum: usize) -> usize {
        1 + self.rng.below(maximum as u64) as usize
    }

    fn value(&mut self, operation_index: u64) -> Vec<u8> {
        let selector = self.rng.below(10_000);
        let size = if selector < 7_000 {
            32 + self.rng.below(224) as usize
        } else {
            512 + self.rng.below(4 * 1024) as usize
        };
        patterned_value(operation_index, size)
    }

    fn large_value_probe_operation(&mut self, operation_index: u64) -> Option<GeneratedOperation> {
        let step = operation_index % 128;
        if step >= 8 {
            return None;
        }
        let cycle = operation_index / 128;
        let tenant = TenantId::new((cycle % self.config.tenant_count) + 1);
        let probe = (cycle % u64::from(self.config.large_probe_key_count)) as u32;
        let key = self.large_probe_key(tenant, probe);
        let value_size = match step {
            0 => 64,
            1 => 511,
            2 => 513,
            3 => 768 * 1024,
            4 => 1024 * 1024,
            5 => 4 * 1024 * 1024,
            6 => 64,
            7 => 2 * 1024 * 1024,
            _ => unreachable!(),
        };
        Some(match step {
            0..=5 | 7 => GeneratedOperation::Put {
                tenant,
                key,
                value: patterned_value(operation_index, value_size),
            },
            6 => GeneratedOperation::Delete { tenant, key },
            _ => unreachable!(),
        })
    }

    fn regular_value_for_key(&mut self, operation_index: u64, key: &DocumentKey) -> Vec<u8> {
        if key
            .pk
            .as_bytes()
            .windows(5)
            .any(|window| window == b"-hot-")
        {
            patterned_value(operation_index, 64)
        } else {
            self.value(operation_index)
        }
    }

    fn value_for_key(&mut self, operation_index: u64, key: &DocumentKey) -> Vec<u8> {
        // Large values are only generated by the finite probe schedule. This
        // keeps overflow coverage while preventing value-size randomness from
        // creating a second persistent keyspace.
        self.regular_value_for_key(operation_index, key)
    }
}

fn patterned_value(operation_index: u64, size: usize) -> Vec<u8> {
    let prefix = format!("soak_operation_id={operation_index};payload=");
    let mut value = Vec::with_capacity(size.max(prefix.len()));
    value.extend_from_slice(prefix.as_bytes());
    while value.len() < size {
        value.push((operation_index as u8).wrapping_add(value.len() as u8));
    }
    value.truncate(size);
    value
}

/// Values at or below this size are retained in the model for convenient
/// diagnostics. Larger values keep only their length and SHA-256 digest.
pub const MODEL_INLINE_VALUE_LIMIT: usize = 512;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExpectedValue {
    length: usize,
    digest: [u8; 32],
    inline: Option<Box<[u8]>>,
}

impl ExpectedValue {
    pub fn from_bytes(value: &[u8]) -> Self {
        let digest = Sha256::digest(value).into();
        Self {
            length: value.len(),
            digest,
            inline: (value.len() <= MODEL_INLINE_VALUE_LIMIT)
                .then(|| value.to_vec().into_boxed_slice()),
        }
    }

    pub const fn length(&self) -> usize {
        self.length
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub fn matches(&self, value: &[u8]) -> bool {
        let digest: [u8; 32] = Sha256::digest(value).into();
        self.length == value.len() && self.digest == digest
    }

    pub fn retained_bytes(&self) -> usize {
        self.inline.as_ref().map_or(0, |value| value.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpectedState {
    Present {
        value: ExpectedValue,
        revision: Revision,
    },
    Missing {
        revision: Revision,
    },
}

impl ExpectedState {
    pub fn present(value: impl AsRef<[u8]>, revision: Revision) -> Self {
        Self::Present {
            value: ExpectedValue::from_bytes(value.as_ref()),
            revision,
        }
    }

    pub const fn missing(revision: Revision) -> Self {
        Self::Missing { revision }
    }

    fn from_revision_state(state: &RevisionState) -> Self {
        match state {
            RevisionState::Present { value, revision } => Self::present(value, *revision),
            RevisionState::Missing { revision } => Self::missing(*revision),
        }
    }

    pub const fn revision(&self) -> Revision {
        match self {
            Self::Present { revision, .. } | Self::Missing { revision } => *revision,
        }
    }

    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing { .. })
    }

    pub fn value(&self) -> Option<&ExpectedValue> {
        match self {
            Self::Present { value, .. } => Some(value),
            Self::Missing { .. } => None,
        }
    }

    fn matches_revision_state(&self, actual: &RevisionState) -> bool {
        match (self, actual) {
            (
                Self::Present { value, revision },
                RevisionState::Present {
                    value: actual_value,
                    revision: actual_revision,
                },
            ) => revision == actual_revision && value.matches(actual_value),
            (
                Self::Missing { revision },
                RevisionState::Missing {
                    revision: actual_revision,
                },
            ) => revision == actual_revision,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedDocument {
    pub key: DocumentKey,
    pub value: ExpectedValue,
    pub revision: Revision,
}

/// A logical model of the committed state. Revisions are copied from actual
/// successful dodb responses; this model never predicts a future revision.
/// Large values are represented by compact fingerprints instead of retaining
/// their complete byte vectors in current state and ABA history.
#[derive(Clone, Debug, Default)]
pub struct ReferenceModel {
    states: BTreeMap<TenantId, BTreeMap<DocumentKey, ExpectedState>>,
    history: BTreeMap<(TenantId, DocumentKey), Vec<ExpectedState>>,
    candidates: BTreeMap<(TenantId, DocumentKey), Vec<ExpectedValue>>,
    delete_candidates: BTreeSet<(TenantId, DocumentKey)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnknownMutationResolution {
    Applied,
    NotApplied,
    Partial,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StateShape {
    Missing,
    Present(ExpectedValue),
}

fn expected_state_shape(state: &ExpectedState) -> StateShape {
    match state {
        ExpectedState::Present { value, .. } => StateShape::Present(value.clone()),
        ExpectedState::Missing { .. } => StateShape::Missing,
    }
}

fn actual_state_shape(state: &RevisionState) -> StateShape {
    match state {
        RevisionState::Present { value, .. } => {
            StateShape::Present(ExpectedValue::from_bytes(value))
        }
        RevisionState::Missing { .. } => StateShape::Missing,
    }
}

fn operation_effects(
    operation: &GeneratedOperation,
    keys: &[DocumentKey],
) -> Result<Vec<(usize, StateShape)>, String> {
    let mut effects = Vec::new();
    match operation {
        GeneratedOperation::Put { key, value, .. } => {
            let index = keys
                .iter()
                .position(|candidate| candidate == key)
                .ok_or_else(|| {
                    "unknown put key was not included in reconciliation set".to_owned()
                })?;
            effects.push((index, StateShape::Present(ExpectedValue::from_bytes(value))));
        }
        GeneratedOperation::Delete { key, .. } => {
            let index = keys
                .iter()
                .position(|candidate| candidate == key)
                .ok_or_else(|| {
                    "unknown delete key was not included in reconciliation set".to_owned()
                })?;
            effects.push((index, StateShape::Missing));
        }
        GeneratedOperation::Transact { mutations, .. } => {
            for mutation in mutations {
                let index = keys
                    .iter()
                    .position(|candidate| candidate == &mutation.key)
                    .ok_or_else(|| {
                        "unknown transaction key was not included in reconciliation set".to_owned()
                    })?;
                let shape = match mutation.kind {
                    PlannedMutationKind::Put => {
                        StateShape::Present(ExpectedValue::from_bytes(&mutation.value))
                    }
                    PlannedMutationKind::Delete => StateShape::Missing,
                };
                effects.push((index, shape));
            }
        }
        _ => return Err("non-mutation operation cannot have an unknown outcome".to_owned()),
    }
    Ok(effects)
}

/// Reconciles a complete overlapping set of unknown mutations against one
/// final read. Presence and values are compared here; revisions are applied
/// to the model only after the final semantic outcome has been accepted.
pub fn reconcile_unknown_operations(
    operations: &[GeneratedOperation],
    keys: &[DocumentKey],
    current: &[ExpectedState],
    actual: &[RevisionState],
) -> Result<UnknownMutationResolution, String> {
    if keys.len() != current.len() || keys.len() != actual.len() || operations.is_empty() {
        return Err("unknown reconciliation input lengths are inconsistent".to_owned());
    }
    let current_shapes = current.iter().map(expected_state_shape).collect::<Vec<_>>();
    let actual_shapes = actual.iter().map(actual_state_shape).collect::<Vec<_>>();
    let effects = operations
        .iter()
        .map(|operation| operation_effects(operation, keys))
        .collect::<Result<Vec<_>, _>>()?;

    if current
        .iter()
        .zip(actual)
        .all(|(expected, actual)| expected.matches_revision_state(actual))
    {
        return Ok(UnknownMutationResolution::NotApplied);
    }

    // Single-key mutations can commit in any order and overwrite one another.
    // The final state only needs to be one of the states that could be left by
    // the complete pending set, not an attribution of every intermediate
    // commit to one response.
    if effects.iter().all(|effect| effect.len() == 1) {
        for key_index in 0..keys.len() {
            let possible = effects
                .iter()
                .filter_map(|effect| (effect[0].0 == key_index).then_some(&effect[0].1));
            if actual_shapes[key_index] != current_shapes[key_index]
                && !possible
                    .into_iter()
                    .any(|shape| shape == &actual_shapes[key_index])
            {
                return Ok(UnknownMutationResolution::Partial);
            }
        }
        return Ok(UnknownMutationResolution::Applied);
    }

    // Transactions are explored as atomic events. The state-space is bounded
    // because the generator gives crash-phase multi-key transactions unique
    // keys; the fallback still retains the strict all-or-none shape check for
    // unusually large pending components.
    if operations.len() > 12 {
        let all_applied = effects.iter().all(|effect| {
            effect
                .iter()
                .all(|(index, shape)| actual_shapes[*index] == *shape)
        });
        return if all_applied {
            Ok(UnknownMutationResolution::Applied)
        } else {
            Ok(UnknownMutationResolution::Partial)
        };
    }
    let mut frontier = vec![(0u16, current_shapes.clone())];
    let mut visited = HashSet::new();
    visited.insert((0u16, current_shapes));
    let full_mask = (1u16 << operations.len()) - 1;
    while let Some((mask, shapes)) = frontier.pop() {
        if mask == full_mask && shapes == actual_shapes {
            return Ok(UnknownMutationResolution::Applied);
        }
        for (operation_index, _) in effects.iter().enumerate() {
            let bit = 1u16 << operation_index;
            if mask & bit != 0 {
                continue;
            }
            let next_mask = mask | bit;
            let skipped = (next_mask, shapes.clone());
            if visited.insert(skipped.clone()) {
                frontier.push(skipped);
            }
            let mut applied = shapes.clone();
            for (index, shape) in &effects[operation_index] {
                applied[*index] = shape.clone();
            }
            let candidate = (next_mask, applied);
            if visited.insert(candidate.clone()) {
                frontier.push(candidate);
            }
        }
    }
    Ok(UnknownMutationResolution::Partial)
}

pub fn classify_unknown_mutation(
    operation: &GeneratedOperation,
    _before: &[ExpectedState],
    actual: &[RevisionState],
    current: &[ExpectedState],
) -> UnknownMutationResolution {
    let keys = operation.keys().into_iter().cloned().collect::<Vec<_>>();
    reconcile_unknown_operations(std::slice::from_ref(operation), &keys, current, actual)
        .unwrap_or(UnknownMutationResolution::Partial)
}

impl ReferenceModel {
    pub fn state(&self, tenant: TenantId, key: &DocumentKey) -> ExpectedState {
        self.states
            .get(&tenant)
            .and_then(|states| states.get(key))
            .cloned()
            .unwrap_or_else(|| ExpectedState::missing(Revision::ZERO))
    }

    pub fn apply(&mut self, tenant: TenantId, key: DocumentKey, state: RevisionState) {
        let state = ExpectedState::from_revision_state(&state);
        let history = self.history.entry((tenant, key.clone())).or_default();
        if history.last() != Some(&state) {
            history.push(state.clone());
            if history.len() > 64 {
                history.remove(0);
            }
        }
        let states = self.states.entry(tenant).or_default();
        let should_publish = states
            .get(&key)
            .is_none_or(|current| current.revision() <= state.revision());
        if should_publish {
            states.insert(key, state);
        }
    }

    pub fn apply_put(
        &mut self,
        tenant: TenantId,
        key: DocumentKey,
        value: Vec<u8>,
        revision: Revision,
    ) {
        self.apply(tenant, key, RevisionState::present(value, revision));
    }

    pub fn apply_delete(&mut self, tenant: TenantId, key: DocumentKey, revision: Revision) {
        self.apply(tenant, key, RevisionState::missing(revision));
    }

    pub fn record_expected_put(&mut self, tenant: TenantId, key: DocumentKey, value: &[u8]) {
        let candidates = self.candidates.entry((tenant, key)).or_default();
        let expected = ExpectedValue::from_bytes(value);
        if candidates.last() != Some(&expected) {
            candidates.push(expected);
            if candidates.len() > 16 {
                candidates.remove(0);
            }
        }
    }

    pub fn record_expected_delete(&mut self, tenant: TenantId, key: DocumentKey) {
        self.delete_candidates.insert((tenant, key));
    }

    pub fn knows_state(&self, tenant: TenantId, key: &DocumentKey, state: &RevisionState) -> bool {
        if matches!(state, RevisionState::Missing { revision } if *revision == Revision::ZERO) {
            return true;
        }
        self.history
            .get(&(tenant, key.clone()))
            .is_some_and(|history| {
                history
                    .iter()
                    .any(|known| known.matches_revision_state(state))
            })
            || self.state(tenant, key).matches_revision_state(state)
            || matches!(state, RevisionState::Present { value, .. }
                if self
                    .candidates
                    .get(&(tenant, key.clone()))
                    .is_some_and(|candidates| candidates.iter().any(|candidate| candidate.matches(value))))
            || matches!(state, RevisionState::Missing { .. }
                if self.delete_candidates.contains(&(tenant, key.clone())))
    }

    pub fn knows_revision(&self, tenant: TenantId, key: &DocumentKey, revision: Revision) -> bool {
        self.history
            .get(&(tenant, key.clone()))
            .is_some_and(|history| history.iter().any(|state| state.revision() == revision))
            || self.state(tenant, key).revision() == revision
    }

    pub fn matches_state(
        &self,
        tenant: TenantId,
        key: &DocumentKey,
        state: &RevisionState,
    ) -> bool {
        self.knows_state(tenant, key, state)
    }

    pub fn scan(
        &self,
        tenant: TenantId,
        cursor: Option<&DocumentKey>,
        limit: usize,
    ) -> Vec<ExpectedDocument> {
        self.states
            .get(&tenant)
            .into_iter()
            .flat_map(|states| states.iter())
            .filter_map(|(key, state)| {
                if cursor.is_some_and(|cursor| key <= cursor) {
                    return None;
                }
                match state {
                    ExpectedState::Present { value, revision } => Some(ExpectedDocument {
                        key: key.clone(),
                        value: value.clone(),
                        revision: *revision,
                    }),
                    ExpectedState::Missing { .. } => None,
                }
            })
            .take(limit)
            .collect()
    }

    pub fn query(
        &self,
        tenant: TenantId,
        pk: &PrimaryKey,
        cursor: Option<&dodb_core::SortKey>,
        limit: usize,
    ) -> Vec<ExpectedDocument> {
        self.scan(tenant, None, usize::MAX)
            .into_iter()
            .filter(|document| {
                &document.key.pk == pk && cursor.is_none_or(|cursor| &document.key.sk > cursor)
            })
            .take(limit)
            .collect()
    }

    pub fn documents(&self, tenant: TenantId) -> Vec<ExpectedDocument> {
        self.scan(tenant, None, usize::MAX)
    }

    pub fn keys(&self, tenant: TenantId) -> Vec<DocumentKey> {
        self.states
            .get(&tenant)
            .into_iter()
            .flat_map(|states| states.keys().cloned())
            .collect()
    }

    pub fn total_key_count(&self) -> usize {
        self.states.values().map(BTreeMap::len).sum()
    }

    pub fn all_documents(&self) -> Vec<(TenantId, ExpectedDocument)> {
        self.tenants()
            .flat_map(|(tenant, documents)| {
                documents
                    .into_iter()
                    .map(move |document| (tenant, document))
            })
            .collect()
    }

    pub fn tenants(&self) -> impl Iterator<Item = (TenantId, Vec<ExpectedDocument>)> + '_ {
        self.states
            .keys()
            .copied()
            .map(|tenant| (tenant, self.scan(tenant, None, usize::MAX)))
    }

    pub fn known_key_count(&self) -> usize {
        self.states.values().map(BTreeMap::len).sum()
    }

    pub fn retained_value_bytes(&self) -> usize {
        let current = self
            .states
            .values()
            .flat_map(BTreeMap::values)
            .filter_map(ExpectedState::value)
            .map(ExpectedValue::retained_bytes)
            .sum::<usize>();
        let history = self
            .history
            .values()
            .flat_map(|states| states.iter().filter_map(ExpectedState::value))
            .map(ExpectedValue::retained_bytes)
            .sum::<usize>();
        let candidates = self
            .candidates
            .values()
            .flat_map(|values| values.iter())
            .map(ExpectedValue::retained_bytes)
            .sum::<usize>();
        current + history + candidates
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationRecord {
    pub index: u64,
    pub global_index: u64,
    pub phase: String,
    pub phase_index: u64,
    pub seed: u64,
    pub workload_config: WorkloadConfig,
    pub elapsed_ms: u128,
    pub tenant: u64,
    pub operation: String,
    pub generated_operation: String,
    pub status: String,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedMetricsSample {
    pub timestamp_ms: u64,
    pub cycle: u64,
    pub sequence: usize,
    pub allocator: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedEventSummary {
    pub checkpoint_successes: u64,
    pub checkpoint_failures: Vec<String>,
    pub invariant_checks: u64,
    pub invariant_passes: u64,
    pub invariant_failures: Vec<String>,
    pub metrics: Vec<ParsedMetricsSample>,
}

/// Parses a completed child event log. A hard-killed child may leave one
/// unterminated final line; callers may explicitly tolerate only that case.
pub fn parse_event_lines(
    contents: &str,
    tolerate_trailing_partial: bool,
) -> Result<ParsedEventSummary, String> {
    let mut summary = ParsedEventSummary::default();
    let trailing_partial = !contents.ends_with('\n');
    for (sequence, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            return Err(format!("event line {sequence} is empty"));
        }
        let value = match serde_json::from_str::<Value>(line) {
            Ok(value) => value,
            Err(error)
                if tolerate_trailing_partial
                    && trailing_partial
                    && sequence + 1 == contents.lines().count() =>
            {
                let _ = error;
                continue;
            }
            Err(error) => return Err(format!("malformed event line {sequence}: {error}")),
        };
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("event line {sequence} has no kind"))?;
        let timestamp_ms = value
            .get("timestamp_ms")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let cycle = value
            .get("cycle")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        match kind {
            "ready" => {}
            "metrics" => {
                let allocator = match value.get("allocator") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(allocator)) => {
                        Some(serde_json::from_str(allocator).map_err(|error| {
                            format!("invalid allocator JSON on event line {sequence}: {error}")
                        })?)
                    }
                    Some(_) => {
                        return Err(format!(
                            "metrics allocator on event line {sequence} is not a JSON string"
                        ));
                    }
                };
                summary.metrics.push(ParsedMetricsSample {
                    timestamp_ms,
                    cycle,
                    sequence,
                    allocator,
                });
            }
            "checkpoint" => {
                summary.checkpoint_successes += 1;
                summary.invariant_checks += 1;
                let invariant_error = match value.get("invariant_error") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(error)) => Some(error.as_str()),
                    Some(_) => {
                        return Err(format!(
                            "checkpoint invariant_error on event line {sequence} is not a string"
                        ));
                    }
                };
                if let Some(error) = invariant_error {
                    summary.invariant_failures.push(error.to_owned());
                } else {
                    summary.invariant_passes += value
                        .get("invariant_shards")
                        .and_then(Value::as_u64)
                        .unwrap_or_default();
                }
            }
            "checkpoint_error" => {
                let error = value.get("error").and_then(Value::as_str).ok_or_else(|| {
                    format!("checkpoint_error on event line {sequence} has no error")
                })?;
                summary.checkpoint_failures.push(error.to_owned());
            }
            other => return Err(format!("unknown event kind {other:?} on line {sequence}")),
        }
    }
    Ok(summary)
}

#[derive(Clone, Debug)]
pub struct RecentOperations {
    capacity: usize,
    entries: VecDeque<OperationRecord>,
}

impl RecentOperations {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: VecDeque::with_capacity(capacity.max(1)),
        }
    }

    pub fn push(&mut self, record: OperationRecord) {
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(record);
    }

    pub fn as_vec(&self) -> Vec<OperationRecord> {
        self.entries.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LatencyStats {
    samples: Vec<u64>,
    count: u64,
    maximum: u64,
}

impl LatencyStats {
    pub fn record(&mut self, nanos: u64) {
        self.count = self.count.saturating_add(1);
        self.maximum = self.maximum.max(nanos);
        const SAMPLE_LIMIT: usize = 200_000;
        if self.samples.len() < SAMPLE_LIMIT {
            self.samples.push(nanos);
        } else {
            self.samples[(self.count as usize) % SAMPLE_LIMIT] = nanos;
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn maximum(&self) -> u64 {
        self.maximum
    }

    pub fn percentile(&self, percentile: f64) -> u64 {
        percentile_value(&self.samples, percentile)
    }
}

pub fn percentile_value(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let clamped = percentile.clamp(0.0, 100.0) / 100.0;
    let position = ((sorted.len() - 1) as f64 * clamped).round() as usize;
    sorted[position]
}

#[derive(Clone, Debug, Serialize)]
pub struct ResourceSample {
    pub phase: String,
    pub elapsed_ms: u128,
    pub post_quiescence: bool,
    pub rss_bytes: Option<u64>,
    pub virtual_bytes: Option<u64>,
    pub fd_count: Option<u64>,
    pub thread_count: Option<u64>,
    pub wal_bytes: u64,
    pub database_bytes: u64,
    pub active_connections: Option<u64>,
    pub active_streams: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrendDiagnostic {
    pub metric: String,
    pub warning: bool,
    pub warmup_ignored: bool,
    pub first_late_window: u64,
    pub last_late_window: u64,
    pub slope_per_second: f64,
    pub detail: String,
}

pub fn trend_diagnostic(metric: &str, samples: &[(f64, u64)], warmup: Duration) -> TrendDiagnostic {
    if samples.len() < 6 {
        return TrendDiagnostic {
            metric: metric.to_owned(),
            warning: false,
            warmup_ignored: false,
            first_late_window: samples.first().map_or(0, |sample| sample.1),
            last_late_window: samples.last().map_or(0, |sample| sample.1),
            slope_per_second: 0.0,
            detail: "insufficient samples".to_owned(),
        };
    }
    let cutoff = samples.first().map_or(0.0, |sample| sample.0) + warmup.as_secs_f64();
    let late = samples
        .iter()
        .filter(|sample| sample.0 >= cutoff)
        .collect::<Vec<_>>();
    if late.len() < 3 {
        return TrendDiagnostic {
            metric: metric.to_owned(),
            warning: false,
            warmup_ignored: true,
            first_late_window: late.first().map_or(0, |sample| sample.1),
            last_late_window: late.last().map_or(0, |sample| sample.1),
            slope_per_second: 0.0,
            detail: "insufficient post-warmup samples".to_owned(),
        };
    }
    let window = (late.len() / 3).max(1);
    let means = [
        mean_window(&late[..window]),
        mean_window(
            &late[late.len().saturating_sub(2 * window)..late.len().saturating_sub(window)],
        ),
        mean_window(&late[late.len().saturating_sub(window)..]),
    ];
    let first = means[0].round() as u64;
    let last = means[2].round() as u64;
    let growth = last.saturating_sub(first);
    let threshold = first.max(4) / 4;
    let warning = means[1] >= means[0] && means[2] >= means[1] && growth > threshold;
    let elapsed =
        late.last().map_or(1.0, |sample| sample.0) - late.first().map_or(0.0, |sample| sample.0);
    TrendDiagnostic {
        metric: metric.to_owned(),
        warning,
        warmup_ignored: true,
        first_late_window: first,
        last_late_window: last,
        slope_per_second: if elapsed > 0.0 {
            (last as f64 - first as f64) / elapsed
        } else {
            0.0
        },
        detail: if warning {
            "later windows continue to rise after warm-up".to_owned()
        } else {
            "no sustained late-window growth detected".to_owned()
        },
    }
}

fn mean_window(samples: &[&(f64, u64)]) -> f64 {
    if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|sample| sample.1 as f64).sum::<f64>() / samples.len() as f64
    }
}

impl fmt::Display for OperationKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
        DocumentKey::new(pk.to_vec(), sk.to_vec())
    }

    #[test]
    fn same_seed_produces_same_logical_sequence() {
        let config = WorkloadConfig {
            tenant_count: 3,
            hot_key_count: 5,
            wide_key_count: 20,
            ..WorkloadConfig::default()
        };
        let mut left = OperationGenerator::new(7, config.clone()).unwrap();
        let mut right = OperationGenerator::new(7, config).unwrap();
        for index in 0..200 {
            assert_eq!(left.next(index), right.next(index));
        }
    }

    #[test]
    fn phase_local_operation_index_replays_independently_of_global_offset() {
        let config = WorkloadConfig {
            tenant_count: 2,
            ..WorkloadConfig::default()
        };
        let mut first = OperationGenerator::new(7 ^ 0x2222, config.clone()).unwrap();
        let mut replay = OperationGenerator::new(7 ^ 0x2222, config).unwrap();
        for phase_index in 0..32 {
            assert_eq!(
                first.next(phase_index),
                replay.next(phase_index),
                "phase-local operation index {phase_index} did not replay"
            );
        }
    }

    #[test]
    fn bounded_generator_reuses_a_finite_keyspace_and_replays() {
        let config = WorkloadConfig {
            tenant_count: 2,
            hot_key_count: 4,
            wide_key_count: 12,
            bounded_keyspace: true,
            operation_weights: [10, 30, 20, 5, 5, 10, 20],
            max_transaction_mutations: 3,
            ..WorkloadConfig::default()
        };
        let mut first = OperationGenerator::new(41, config.clone()).unwrap();
        let mut replay = OperationGenerator::new(41, config).unwrap();
        let mut keys = HashSet::new();
        let mut key_kinds = BTreeMap::<DocumentKey, HashSet<OperationKind>>::new();
        let mut transaction_count = 0;
        for index in 0..2_000 {
            let operation = first.next(index);
            assert_eq!(operation, replay.next(index));
            for key in operation.keys() {
                keys.insert(key.clone());
                key_kinds
                    .entry(key.clone())
                    .or_default()
                    .insert(operation.kind());
            }
            if let GeneratedOperation::Transact { mutations, .. } = &operation {
                transaction_count += 1;
                assert!(mutations.len() <= 3);
                let unique = mutations
                    .iter()
                    .map(|mutation| &mutation.key)
                    .collect::<HashSet<_>>();
                assert_eq!(unique.len(), mutations.len());
            }
        }
        assert!(keys.len() <= 2 * (4 + 12 + 16 + 2));
        assert!(transaction_count > 0);
        assert!(key_kinds.values().any(|kinds| {
            kinds.contains(&OperationKind::Put) && kinds.contains(&OperationKind::Delete)
        }));
    }

    #[test]
    fn growth_contention_and_crash_configs_have_explicit_finite_key_bounds() {
        let configurations = [
            WorkloadConfig {
                tenant_count: 2,
                growth_target_key_count: 24,
                growth_fill_operations: 48,
                bounded_keyspace: false,
                ..WorkloadConfig::default()
            },
            WorkloadConfig {
                tenant_count: 2,
                bounded_keyspace: true,
                wide_key_count: 8,
                ..WorkloadConfig::default()
            },
            WorkloadConfig {
                tenant_count: 2,
                bounded_keyspace: true,
                wide_key_count: 6,
                ..WorkloadConfig::default()
            },
        ];
        for config in configurations {
            let upper_bound = config.maximum_generated_key_count();
            let mut generator = OperationGenerator::new(1, config.clone()).unwrap();
            let mut keys = HashSet::new();
            for index in 0..20_000 {
                for key in generator.next(index).keys() {
                    keys.insert(key.clone());
                }
            }
            assert!(keys.len() <= upper_bound, "generated {keys:?}");
        }
    }

    #[test]
    fn bounded_workload_contains_put_delete_updates_and_transactions() {
        let config = WorkloadConfig {
            tenant_count: 2,
            hot_key_count: 3,
            wide_key_count: 8,
            bounded_keyspace: true,
            operation_weights: [10, 30, 25, 0, 0, 0, 35],
            max_transaction_mutations: 3,
            ..WorkloadConfig::default()
        };
        let mut generator = OperationGenerator::new(9, config).unwrap();
        let mut puts = 0;
        let mut deletes = 0;
        let mut transactions = 0;
        let mut put_keys = HashSet::new();
        let mut deleted_keys = HashSet::new();
        for index in 0..4_000 {
            match generator.next(index) {
                GeneratedOperation::Put { key, .. } => {
                    puts += 1;
                    put_keys.insert(key);
                }
                GeneratedOperation::Delete { key, .. } => {
                    deletes += 1;
                    deleted_keys.insert(key);
                }
                GeneratedOperation::Transact { .. } => transactions += 1,
                _ => {}
            }
        }
        assert!(puts > 0 && deletes > 0 && transactions > 0);
        assert!(put_keys.intersection(&deleted_keys).next().is_some());
    }

    #[test]
    fn large_value_probe_keys_are_reused() {
        let config = WorkloadConfig {
            tenant_count: 3,
            large_probe_key_count: 2,
            ..WorkloadConfig::default()
        };
        let mut generator = OperationGenerator::new(3, config.clone()).unwrap();
        let mut probes = HashSet::new();
        for index in 0..20_000 {
            let operation = generator.next(index);
            let is_probe = |key: &DocumentKey| {
                key.pk
                    .as_bytes()
                    .windows(b"-large-probe-".len())
                    .any(|window| window == b"-large-probe-")
            };
            match operation {
                GeneratedOperation::Put { key, value, .. } if is_probe(&key) => {
                    probes.insert(key);
                    assert!(value.len() <= 4 * 1024 * 1024);
                }
                GeneratedOperation::Delete { key, .. } if is_probe(&key) => {
                    probes.insert(key);
                }
                _ => {}
            }
        }
        assert_eq!(
            probes.len(),
            (config.tenant_count * u64::from(config.large_probe_key_count)) as usize
        );
    }

    #[test]
    fn model_preserves_missing_delete_revision_and_aba_history() {
        let tenant = TenantId::new(1);
        let document = key(b"p", b"s");
        let mut model = ReferenceModel::default();
        assert_eq!(
            model.state(tenant, &document),
            ExpectedState::missing(Revision::ZERO)
        );
        model.apply_put(tenant, document.clone(), b"a".to_vec(), Revision::new(4));
        model.apply_delete(tenant, document.clone(), Revision::new(5));
        assert_eq!(
            model.state(tenant, &document),
            ExpectedState::missing(Revision::new(5))
        );
        assert!(model.knows_state(tenant, &document, &RevisionState::missing(Revision::ZERO)));
        assert!(model.knows_state(
            tenant,
            &document,
            &RevisionState::present(b"a".to_vec(), Revision::new(4))
        ));
    }

    #[test]
    fn compact_expected_values_detect_changed_contents_and_lengths() {
        let expected = ExpectedValue::from_bytes(&vec![7; MODEL_INLINE_VALUE_LIMIT + 128]);
        assert!(expected.matches(&vec![7; MODEL_INLINE_VALUE_LIMIT + 128]));
        assert!(!expected.matches(&vec![8; MODEL_INLINE_VALUE_LIMIT + 128]));
        assert!(!expected.matches(&vec![7; MODEL_INLINE_VALUE_LIMIT + 127]));

        let tenant = TenantId::new(1);
        let document = key(b"compact", b"value");
        let mut model = ReferenceModel::default();
        model.apply_put(
            tenant,
            document.clone(),
            vec![7; MODEL_INLINE_VALUE_LIMIT + 128],
            Revision::new(4),
        );
        assert_eq!(model.retained_value_bytes(), 0);
        assert!(model.knows_state(
            tenant,
            &document,
            &RevisionState::present(vec![7; MODEL_INLINE_VALUE_LIMIT + 128], Revision::new(4))
        ));
        assert!(!model.knows_state(
            tenant,
            &document,
            &RevisionState::present(vec![8; MODEL_INLINE_VALUE_LIMIT + 128], Revision::new(4))
        ));
    }

    #[test]
    fn unknown_multi_key_mutation_accepts_only_all_or_none() {
        let tenant = TenantId::new(1);
        let first = key(b"p", b"a");
        let second = key(b"p", b"b");
        let operation = GeneratedOperation::Transact {
            tenant,
            mutations: vec![
                PlannedMutation {
                    key: first.clone(),
                    kind: PlannedMutationKind::Put,
                    value: b"first".to_vec(),
                },
                PlannedMutation {
                    key: second.clone(),
                    kind: PlannedMutationKind::Put,
                    value: b"second".to_vec(),
                },
            ],
        };
        let before = vec![ExpectedState::missing(Revision::ZERO); 2];
        let current = before.clone();
        let applied = vec![
            RevisionState::present(b"first".to_vec(), Revision::new(10)),
            RevisionState::present(b"second".to_vec(), Revision::new(10)),
        ];
        assert_eq!(
            classify_unknown_mutation(&operation, &before, &applied, &current),
            UnknownMutationResolution::Applied
        );
        assert_eq!(
            classify_unknown_mutation(
                &operation,
                &before,
                &[applied[0].clone(), RevisionState::missing(Revision::ZERO),],
                &current,
            ),
            UnknownMutationResolution::Partial
        );
    }

    #[test]
    fn overlapping_unknown_puts_reconcile_to_the_final_writer() {
        let tenant = TenantId::new(1);
        let document = key(b"p", b"s");
        let operations = vec![
            GeneratedOperation::Put {
                tenant,
                key: document.clone(),
                value: b"a".to_vec(),
            },
            GeneratedOperation::Put {
                tenant,
                key: document.clone(),
                value: b"b".to_vec(),
            },
        ];
        let current = vec![ExpectedState::missing(Revision::ZERO)];
        let actual = vec![RevisionState::present(b"b".to_vec(), Revision::new(8))];
        assert_eq!(
            reconcile_unknown_operations(
                &operations,
                std::slice::from_ref(&document),
                &current,
                &actual
            )
            .unwrap(),
            UnknownMutationResolution::Applied
        );
    }

    #[test]
    fn unknown_put_delete_chains_reconcile_without_attribution() {
        let tenant = TenantId::new(1);
        let document = key(b"p", b"s");
        let put = GeneratedOperation::Put {
            tenant,
            key: document.clone(),
            value: b"a".to_vec(),
        };
        let delete = GeneratedOperation::Delete {
            tenant,
            key: document.clone(),
        };
        let current = vec![ExpectedState::missing(Revision::ZERO)];
        let present = vec![RevisionState::present(b"a".to_vec(), Revision::new(8))];
        assert_eq!(
            reconcile_unknown_operations(
                &[put.clone(), delete.clone()],
                std::slice::from_ref(&document),
                &current,
                &present,
            )
            .unwrap(),
            UnknownMutationResolution::Applied
        );
        let delete_then_put = vec![delete, put];
        assert_eq!(
            reconcile_unknown_operations(
                &delete_then_put,
                std::slice::from_ref(&document),
                &current,
                &present,
            )
            .unwrap(),
            UnknownMutationResolution::Applied
        );
    }

    #[test]
    fn known_later_state_can_hide_an_earlier_unknown() {
        let tenant = TenantId::new(1);
        let document = key(b"p", b"s");
        let operation = GeneratedOperation::Put {
            tenant,
            key: document.clone(),
            value: b"old".to_vec(),
        };
        let current = vec![ExpectedState::present(b"new", Revision::new(10))];
        let actual = vec![RevisionState::present(b"new".to_vec(), Revision::new(10))];
        assert_eq!(
            reconcile_unknown_operations(
                &[operation],
                std::slice::from_ref(&document),
                &current,
                &actual,
            )
            .unwrap(),
            UnknownMutationResolution::NotApplied
        );
    }

    #[test]
    fn normal_phase_unknowns_are_reconciled_before_exact_verification() {
        let tenant = TenantId::new(1);
        let document = key(b"p", b"s");
        let operation = GeneratedOperation::Put {
            tenant,
            key: document.clone(),
            value: b"normal-phase".to_vec(),
        };
        let current = vec![ExpectedState::missing(Revision::ZERO)];
        let actual = vec![RevisionState::present(
            b"normal-phase".to_vec(),
            Revision::new(11),
        )];
        let resolution = reconcile_unknown_operations(
            std::slice::from_ref(&operation),
            std::slice::from_ref(&document),
            &current,
            &actual,
        )
        .unwrap();
        assert_eq!(resolution, UnknownMutationResolution::Applied);
        // The exact verification model is only valid after this state update.
        let mut model = ReferenceModel::default();
        model.apply(tenant, document.clone(), actual[0].clone());
        assert_eq!(
            model.state(tenant, &document),
            ExpectedState::present(b"normal-phase", Revision::new(11))
        );
    }

    #[test]
    fn atomic_unknown_transaction_accepts_all_or_none_and_rejects_partial() {
        let tenant = TenantId::new(1);
        let first = key(b"p", b"a");
        let second = key(b"p", b"b");
        let operation = GeneratedOperation::Transact {
            tenant,
            mutations: vec![
                PlannedMutation {
                    key: first.clone(),
                    kind: PlannedMutationKind::Put,
                    value: b"first".to_vec(),
                },
                PlannedMutation {
                    key: second.clone(),
                    kind: PlannedMutationKind::Put,
                    value: b"second".to_vec(),
                },
            ],
        };
        let keys = vec![first, second];
        let current = vec![ExpectedState::missing(Revision::ZERO); 2];
        let all = vec![
            RevisionState::present(b"first".to_vec(), Revision::new(10)),
            RevisionState::present(b"second".to_vec(), Revision::new(10)),
        ];
        let partial = vec![all[0].clone(), RevisionState::missing(Revision::ZERO)];
        assert_eq!(
            reconcile_unknown_operations(std::slice::from_ref(&operation), &keys, &current, &all)
                .unwrap(),
            UnknownMutationResolution::Applied
        );
        assert_eq!(
            reconcile_unknown_operations(
                std::slice::from_ref(&operation),
                &keys,
                &current,
                &partial,
            )
            .unwrap(),
            UnknownMutationResolution::Partial
        );
    }

    #[test]
    fn checkpoint_error_event_is_not_ignored() {
        let parsed = parse_event_lines(
            "{\"kind\":\"checkpoint\",\"invariant_shards\":2}\n{\"kind\":\"checkpoint_error\",\"error\":\"disk full\"}\n",
            false,
        )
        .unwrap();
        assert_eq!(parsed.checkpoint_successes, 1);
        assert_eq!(parsed.checkpoint_failures, vec!["disk full"]);
    }

    #[test]
    fn checkpoint_and_invariant_failures_are_distinguished() {
        let parsed = parse_event_lines(
            "{\"kind\":\"checkpoint\",\"invariant_shards\":0,\"invariant_error\":\"bad page\"}\n{\"kind\":\"checkpoint_error\",\"error\":\"io\"}\n",
            false,
        )
        .unwrap();
        assert_eq!(parsed.checkpoint_successes, 1);
        assert_eq!(parsed.checkpoint_failures, vec!["io"]);
        assert_eq!(parsed.invariant_failures, vec!["bad page"]);
    }

    #[test]
    fn malformed_event_records_fail_unless_the_final_line_is_a_crash_partial() {
        assert!(parse_event_lines("{\"kind\":\"metrics\"}\nnot-json\n", false).is_err());
        assert!(parse_event_lines("{\"kind\":\"metrics\"}\nnot-json", true).is_ok());
        assert!(parse_event_lines("{\"kind\":\"metrics\"}\nnot-json\n", true).is_err());
    }

    #[test]
    fn percentile_and_recent_operations_are_bounded() {
        assert_eq!(percentile_value(&[1, 2, 3, 4, 5], 50.0), 3);
        let mut recent = RecentOperations::new(2);
        for index in 0..5 {
            recent.push(OperationRecord {
                index,
                global_index: index,
                phase: "test".to_owned(),
                phase_index: index,
                seed: 0,
                workload_config: WorkloadConfig::default(),
                elapsed_ms: 0,
                tenant: 1,
                operation: "get".to_owned(),
                generated_operation: "Get".to_owned(),
                status: "ok".to_owned(),
            });
        }
        assert_eq!(recent.len(), 2);
        assert_eq!(recent.as_vec()[0].index, 3);
    }

    #[test]
    fn trend_ignores_warmup_and_detects_sustained_growth() {
        let samples = (0..12)
            .map(|index| (index as f64, if index < 3 { 10 } else { 10 + index * 10 }))
            .collect::<Vec<_>>();
        assert!(trend_diagnostic("fd", &samples, Duration::from_secs(2)).warning);
    }
}
