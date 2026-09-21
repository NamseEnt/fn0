mod codec;
mod commit_log;
mod engine;
mod server;

pub use codec::{
    decode_document_key, decode_document_value, encode_document_key, encode_document_value,
};
pub use commit_log::{CommitMutation, CommitRecord};
pub use dibi_protocol;
pub use engine::{
    AdminDocument, AdminScanPage, AdminScanRequest, ApplicationWrite, ConditionalWrite,
    ConditionalWriteOutcome, Conflict, DATABASE_FORMAT_VERSION, DibiConfig, DibiEngine, Document,
    StoredDocument, WriteResult,
};
pub use server::{DibiServer, DibiServerConfig, ServerError};

#[derive(Debug, thiserror::Error)]
pub enum DibiError {
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("corrupt key encoding: {0}")]
    CorruptKey(String),
    #[error("corrupt document value: {0}")]
    CorruptValue(String),
    #[error("corrupt metadata: {0}")]
    CorruptMetadata(String),
    #[error("corrupt commit record: {0}")]
    CorruptCommitRecord(String),
    #[error("unsupported database format version {0}")]
    UnsupportedFormatVersion(u32),
    #[error("invalid tenant: {0}")]
    InvalidTenant(String),
    #[error("document version overflow for ({pk:?}, {sk:?})")]
    VersionOverflow { pk: String, sk: String },
    #[error("commit id overflow")]
    CommitIdOverflow,
    #[error("duplicate conditional key ({pk:?}, {sk:?})")]
    DuplicateConditionalKey { pk: String, sk: String },
    #[error("write gate is poisoned")]
    WriteGatePoisoned,
}

pub type Result<T> = std::result::Result<T, DibiError>;
