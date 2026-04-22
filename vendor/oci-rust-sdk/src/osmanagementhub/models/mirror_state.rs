use serde::{Deserialize, Serialize};

/// Mirror overall state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorState {
    #[serde(rename = "UNSYNCED")]
    Unsynced,

    #[serde(rename = "QUEUED")]
    Queued,

    #[serde(rename = "SYNCING")]
    Syncing,

    #[serde(rename = "SYNCED")]
    Synced,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
