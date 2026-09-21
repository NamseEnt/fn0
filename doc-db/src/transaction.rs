use bytes::Bytes;

pub enum ObservedDocument {
    Present { data: Bytes, version: i64 },
    Missing,
}

#[derive(Clone)]
pub(crate) enum TransactItem {
    CheckVersion {
        pk: String,
        sk: String,
        expected_version: i64,
    },
    CheckMissing {
        pk: String,
        sk: String,
    },
    Insert {
        pk: String,
        sk: String,
        data: Vec<u8>,
    },
    Update {
        pk: String,
        sk: String,
        expected_version: i64,
        data: Vec<u8>,
    },
    Delete {
        pk: String,
        sk: String,
        expected_version: i64,
    },
}

pub(crate) struct TransactOutcome {
    pub(crate) conflict: Option<TransactConflict>,
}

pub(crate) struct TransactConflict {
    pub(crate) step_index: usize,
}
