//! Storage-facing codecs and the Phase 1 single-file B+Tree engine.

pub mod blink;
pub mod btree;
pub mod durable_file;
pub mod fault;
pub mod page;
pub mod superblock;
pub mod wal;

pub use blink::{
    BatchPlan, BlinkBatchMetrics, BlinkCheckpointReport, BlinkReadHandle, BlinkSplitMetrics,
    BlinkStore, BlinkVersionedReadMetrics, DependencyEdge, DependencyKind, DependencyMetadata,
    LeafGroupPlan, PhysicalTransactionPlan, PlannedMutation, ProvisionalRevisionToken, RouteHint,
};
pub use btree::{
    AsyncShard, BTreeStore, BatchRequest, BatchResponse, CheckpointReport, CoordinatorConfig,
    CoordinatorMetrics, DatabaseConfig, Document, EngineRequest, EngineResponse, InvariantReport,
    MAX_ENCODED_KEY_SIZE, Mutation, NoWal, PreparedBatch, StorageLimits, StorageMetrics,
};
pub use durable_file::{DurableFile, ProductionFile};
pub use fault::FaultInjector;
pub use page::{
    DecodedPage, PAGE_HEADER_SIZE, PAGE_SIZE, PageHeader, PageType, decode_page, decode_page_at,
    encode_page,
};
pub use superblock::{
    SUPERBLOCK_FORMAT_VERSION, SelectedSuperblock, Superblock, SuperblockSlot, choose_superblock,
    decode_superblock, encode_superblock,
};
pub use wal::{
    CommittedWalBatch, WAL_FORMAT_VERSION, WAL_HEADER_SIZE, WAL_MAGIC, WalAppendReport, WalCommit,
    WalIdentity, WalLog, WalMetrics, WalPageImage, WalPageImageFormat, WalRecordType,
    WalScanReport,
};
