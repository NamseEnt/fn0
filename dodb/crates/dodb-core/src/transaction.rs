use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{DocumentKey, Lsn, ObservedState, Revision};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteIntent {
    Put(Vec<u8>),
    Delete,
}

/// A point-key predicate evaluated at the transaction commit serialization
/// point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionCondition {
    RevisionEquals {
        key: DocumentKey,
        expected_revision: Revision,
    },
    Exists {
        key: DocumentKey,
    },
    NotExists {
        key: DocumentKey,
    },
}

impl TransactionCondition {
    pub fn key(&self) -> &DocumentKey {
        match self {
            Self::RevisionEquals { key, .. } | Self::Exists { key } | Self::NotExists { key } => {
                key
            }
        }
    }

    pub fn expectation(&self) -> ConditionExpectation {
        match self {
            Self::RevisionEquals {
                expected_revision, ..
            } => ConditionExpectation::RevisionEquals(*expected_revision),
            Self::Exists { .. } => ConditionExpectation::Exists,
            Self::NotExists { .. } => ConditionExpectation::NotExists,
        }
    }
}

/// The expected part of a structured transaction conflict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionExpectation {
    RevisionEquals(Revision),
    Exists,
    NotExists,
}

/// A mutation applied atomically with all other mutations in a request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionMutation {
    Put { key: DocumentKey, value: Vec<u8> },
    Delete { key: DocumentKey },
}

impl TransactionMutation {
    pub fn key(&self) -> &DocumentKey {
        match self {
            Self::Put { key, .. } | Self::Delete { key } => key,
        }
    }
}

/// The single-shot semantic primitive for an optimistic atomic write.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransactionRequest {
    pub conditions: Vec<TransactionCondition>,
    pub mutations: Vec<TransactionMutation>,
}

impl TransactionRequest {
    pub fn new(conditions: Vec<TransactionCondition>, mutations: Vec<TransactionMutation>) -> Self {
        Self {
            conditions,
            mutations,
        }
    }

    /// Rejects ambiguous requests before any storage preparation takes place.
    pub fn validate(&self) -> crate::Result<()> {
        if self.conditions.is_empty() && self.mutations.is_empty() {
            return Err(crate::Error::invalid_request(
                "a transaction must contain at least one condition or mutation",
            ));
        }

        let mut mutation_keys = BTreeSet::new();
        for mutation in &self.mutations {
            if !mutation_keys.insert(mutation.key().clone()) {
                return Err(crate::Error::invalid_request(
                    "a transaction contains multiple mutations for one key",
                ));
            }
        }

        let mut condition_keys = BTreeSet::new();
        for condition in &self.conditions {
            if !condition_keys.insert(condition.key().clone()) {
                return Err(crate::Error::invalid_request(
                    "a transaction contains multiple conditions for one key",
                ));
            }
        }
        Ok(())
    }
}

/// The committed identity returned for one successful logical transaction.
///
/// Condition-only transactions successfully validate their read predicates
/// without creating a logical commit, so their commit identity is `None`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionResult {
    pub commit_lsn: Option<Lsn>,
}

/// Structured expected-vs-actual information for an optimistic conflict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionConflict {
    pub key: DocumentKey,
    pub expected: ConditionExpectation,
    pub actual: ObservedState,
}

impl fmt::Display for TransactionConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "key {:?}: expected {:?}, actual {:?}",
            self.key, self.expected, self.actual
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReadSet(BTreeMap<DocumentKey, Revision>);

impl ReadSet {
    pub fn record(&mut self, key: DocumentKey, revision: Revision) {
        self.0.entry(key).or_insert(revision);
    }

    pub fn get(&self, key: &DocumentKey) -> Option<Revision> {
        self.0.get(key).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DocumentKey, &Revision)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WriteSet(BTreeMap<DocumentKey, WriteIntent>);

impl WriteSet {
    pub fn put(&mut self, key: DocumentKey, value: impl Into<Vec<u8>>) {
        self.0.insert(key, WriteIntent::Put(value.into()));
    }

    pub fn delete(&mut self, key: DocumentKey) {
        self.0.insert(key, WriteIntent::Delete);
    }

    pub fn get(&self, key: &DocumentKey) -> Option<&WriteIntent> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DocumentKey, &WriteIntent)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
