use serde::{Deserialize, Serialize};

/// Metadata keys for work request resource metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkRequestResourceMetadataKey {
    #[serde(rename = "IS_DRY_RUN")]
    IsDryRun,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
