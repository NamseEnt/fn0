use bytes::Bytes;
use doc_db_protocol::DocDbRevision;
use std::collections::HashSet;

pub enum ObservedDocument {
    #[doc = "A present document and the revision of its current logical state."]
    Present {
        data: Bytes,
        revision: DocDbRevision,
    },
    #[doc = "A missing document and, when supported by the backend, the revision of its current logical state."]
    Missing { revision: Option<DocDbRevision> },
}

#[derive(Clone)]
pub(crate) enum TransactCondition {
    #[doc = "Compares the current logical key-state revision, whether that state is present or missing."]
    RevisionEquals {
        pk: String,
        sk: String,
        expected_revision: DocDbRevision,
    },
    Exists {
        pk: String,
        sk: String,
    },
    NotExists {
        pk: String,
        sk: String,
    },
}

#[derive(Clone)]
pub(crate) enum TransactMutation {
    Put {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
    },
}

pub(crate) struct TransactRequest {
    pub(crate) conditions: Vec<TransactCondition>,
    pub(crate) mutations: Vec<TransactMutation>,
}

pub(crate) struct TransactOutcome {
    pub(crate) conflict: Option<TransactConflict>,
}

pub(crate) struct TransactConflict {
    pub(crate) condition_index: usize,
}

pub(crate) fn validate_transact_request(request: &TransactRequest) -> anyhow::Result<()> {
    let mut condition_keys = HashSet::with_capacity(request.conditions.len());
    for condition in &request.conditions {
        let (pk, sk) = condition_key(condition);
        if !condition_keys.insert((pk, sk)) {
            anyhow::bail!("duplicate transaction condition key: {pk}/{sk}");
        }
    }

    let mut mutation_keys = HashSet::with_capacity(request.mutations.len());
    for mutation in &request.mutations {
        let (pk, sk) = mutation_key(mutation);
        if !mutation_keys.insert((pk, sk)) {
            anyhow::bail!("duplicate transaction mutation key: {pk}/{sk}");
        }
    }

    Ok(())
}

fn condition_key(condition: &TransactCondition) -> (&str, &str) {
    match condition {
        TransactCondition::RevisionEquals { pk, sk, .. }
        | TransactCondition::Exists { pk, sk }
        | TransactCondition::NotExists { pk, sk } => (pk, sk),
    }
}

fn mutation_key(mutation: &TransactMutation) -> (&str, &str) {
    match mutation {
        TransactMutation::Put { pk, sk, .. } | TransactMutation::Delete { pk, sk } => (pk, sk),
    }
}

pub(crate) fn revision_from_backend(value: i64) -> anyhow::Result<DocDbRevision> {
    u64::try_from(value)
        .map(DocDbRevision::new)
        .map_err(|_| anyhow::anyhow!("backend returned a negative document revision: {value}"))
}

pub(crate) fn revision_to_backend(revision: DocDbRevision) -> anyhow::Result<i64> {
    i64::try_from(revision.value()).map_err(|_| {
        anyhow::anyhow!(
            "backend cannot represent document revision {}",
            revision.value()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_negative_backend_revision() {
        assert!(revision_from_backend(-1).is_err());
    }

    #[test]
    fn rejects_revision_that_does_not_fit_turso_integer() {
        assert!(revision_to_backend(DocDbRevision::new(i64::MAX as u64 + 1)).is_err());
    }

    #[test]
    fn rejects_duplicate_condition_keys() {
        let request = TransactRequest {
            conditions: vec![
                TransactCondition::Exists {
                    pk: "pk".to_string(),
                    sk: "sk".to_string(),
                },
                TransactCondition::NotExists {
                    pk: "pk".to_string(),
                    sk: "sk".to_string(),
                },
            ],
            mutations: vec![],
        };
        assert!(validate_transact_request(&request).is_err());
    }

    #[test]
    fn rejects_duplicate_mutation_keys() {
        let request = TransactRequest {
            conditions: vec![],
            mutations: vec![
                TransactMutation::Put {
                    pk: "pk".to_string(),
                    sk: "sk".to_string(),
                    data: b"one".to_vec(),
                },
                TransactMutation::Delete {
                    pk: "pk".to_string(),
                    sk: "sk".to_string(),
                },
            ],
        };
        assert!(validate_transact_request(&request).is_err());
    }

    #[test]
    fn permits_one_condition_and_one_mutation_for_the_same_key() {
        let request = TransactRequest {
            conditions: vec![TransactCondition::RevisionEquals {
                pk: "pk".to_string(),
                sk: "sk".to_string(),
                expected_revision: DocDbRevision::new(3),
            }],
            mutations: vec![TransactMutation::Put {
                pk: "pk".to_string(),
                sk: "sk".to_string(),
                data: b"updated".to_vec(),
            }],
        };
        assert!(validate_transact_request(&request).is_ok());
    }
}
