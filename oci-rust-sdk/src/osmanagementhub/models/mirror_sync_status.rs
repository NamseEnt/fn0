use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Status summary of the mirror sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSyncStatus {
    /// Total number of software sources that have not yet been synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub unsynced: i64,

    /// Total number of software sources that are queued for sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub queued: i64,

    /// Total number of software sources currently syncing. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub syncing: i64,

    /// Total number of software sources that successfully synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub synced: i64,

    /// Total number of software sources that failed to sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub failed: i64,
}

/// Required fields for MirrorSyncStatus
pub struct MirrorSyncStatusRequired {
    /// Total number of software sources that have not yet been synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub unsynced: i64,

    /// Total number of software sources that are queued for sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub queued: i64,

    /// Total number of software sources currently syncing. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub syncing: i64,

    /// Total number of software sources that successfully synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub synced: i64,

    /// Total number of software sources that failed to sync. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub failed: i64,
}

impl MirrorSyncStatus {
    /// Create a new MirrorSyncStatus with required fields
    pub fn new(required: MirrorSyncStatusRequired) -> Self {
        Self {
            unsynced: required.unsynced,

            queued: required.queued,

            syncing: required.syncing,

            synced: required.synced,

            failed: required.failed,
        }
    }

    /// Set unsynced
    pub fn set_unsynced(mut self, value: i64) -> Self {
        self.unsynced = value;
        self
    }

    /// Set queued
    pub fn set_queued(mut self, value: i64) -> Self {
        self.queued = value;
        self
    }

    /// Set syncing
    pub fn set_syncing(mut self, value: i64) -> Self {
        self.syncing = value;
        self
    }

    /// Set synced
    pub fn set_synced(mut self, value: i64) -> Self {
        self.synced = value;
        self
    }

    /// Set failed
    pub fn set_failed(mut self, value: i64) -> Self {
        self.failed = value;
        self
    }
}
