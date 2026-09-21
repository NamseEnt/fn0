use bytes::Bytes;
use doc_db_protocol::DocDbRevision;

pub enum ObservedDocument {
    Present {
        data: Bytes,
        revision: DocDbRevision,
    },
    Missing {
        revision: Option<DocDbRevision>,
    },
}

#[derive(Clone)]
pub(crate) enum TransactCondition {
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
}
