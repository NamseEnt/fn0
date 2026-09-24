//! Stable semantic contracts shared by the database layers.
//!
//! This crate intentionally contains no storage engine, WAL, networking, or
//! transaction execution machinery.  It defines the values those layers must
//! agree on.

pub mod error;
pub mod identifiers;
pub mod key;
pub mod revision;
pub mod transaction;

pub use error::{Error, Result};
pub use identifiers::{Lsn, PageId, Revision, ShardEpoch, ShardId, TenantId, TxnId};
pub use key::{DocumentKey, KeyCodecError, PrimaryKey, SortKey};
pub use revision::{ObservedState, RevisionState};
pub use transaction::{
    ConditionExpectation, ReadSet, TransactionCondition, TransactionConflict, TransactionMutation,
    TransactionRequest, TransactionResult, WriteIntent, WriteSet,
};
