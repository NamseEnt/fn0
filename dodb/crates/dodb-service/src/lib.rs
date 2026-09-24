use std::future::Future;
use std::pin::Pin;

use dodb_core::{
    DocumentKey, Lsn, PrimaryKey, Result, Revision, RevisionState, SortKey, TenantId,
    TransactionRequest,
};

pub type ServiceFuture<'service> =
    Pin<Box<dyn Future<Output = Result<Response>> + Send + 'service>>;
pub type ShutdownFuture<'service> = Pin<Box<dyn Future<Output = ()> + Send + 'service>>;

/// Per-request resource limits supplied by the protocol/server boundary.
/// Services must enforce the response budget before materializing large read
/// values. The budget includes the encoded response frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionBudget {
    pub max_response_bytes: usize,
}

impl ExecutionBudget {
    pub const fn new(max_response_bytes: usize) -> Self {
        Self { max_response_bytes }
    }
}

pub trait DodbService: Send + Sync {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service>;

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async {})
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    pub key: DocumentKey,
    pub value: Vec<u8>,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Get {
        key: DocumentKey,
    },
    Put {
        key: DocumentKey,
        value: Vec<u8>,
    },
    Delete {
        key: DocumentKey,
    },
    Query {
        pk: PrimaryKey,
        exclusive_after_sk: Option<SortKey>,
        limit: usize,
    },
    Scan {
        exclusive_after_key: Option<DocumentKey>,
        limit: usize,
    },
    Transact {
        request: TransactionRequest,
    },
}

impl Request {
    pub fn operation_name(&self) -> &'static str {
        match self {
            Self::Get { .. } => "get",
            Self::Put { .. } => "put",
            Self::Delete { .. } => "delete",
            Self::Query { .. } => "query",
            Self::Scan { .. } => "scan",
            Self::Transact { .. } => "transact",
        }
    }

    pub fn is_mutation(&self) -> bool {
        match self {
            Self::Put { .. } | Self::Delete { .. } => true,
            Self::Transact { request } => !request.mutations.is_empty(),
            Self::Get { .. } | Self::Query { .. } | Self::Scan { .. } => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Get(RevisionState),
    Put(Revision),
    Delete(Revision),
    Query(Vec<Document>),
    Scan(Vec<Document>),
    Transact(TransactionOutcome),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionOutcome {
    pub commit_lsn: Option<Lsn>,
}

impl TransactionOutcome {
    pub const fn committed(commit_lsn: Lsn) -> Self {
        Self {
            commit_lsn: Some(commit_lsn),
        }
    }

    pub const fn conditions_satisfied() -> Self {
        Self { commit_lsn: None }
    }
}
