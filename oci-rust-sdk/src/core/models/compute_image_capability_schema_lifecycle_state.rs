use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeImageCapabilitySchemaLifecycleState {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DELETED")]
    Deleted,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
