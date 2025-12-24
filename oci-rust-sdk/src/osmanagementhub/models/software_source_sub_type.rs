use serde::{Deserialize, Serialize};

/// Identifies how a custom software source was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareSourceSubType {
    #[serde(rename = "FILTER")]
    Filter,

    #[serde(rename = "MANIFEST")]
    Manifest,

    #[serde(rename = "SNAPSHOT")]
    Snapshot,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
