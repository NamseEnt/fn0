use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitRedundancyMetadataConfiguredRedundancyLevel {
    #[serde(rename = "DEVICE")]
    Device,

    #[serde(rename = "POP")]
    Pop,

    #[serde(rename = "REGION")]
    Region,

    #[serde(rename = "NON_REDUNDANT")]
    NonRedundant,

    #[serde(rename = "PENDING")]
    Pending,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
