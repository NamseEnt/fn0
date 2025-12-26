use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceConsoleConnectionLifecycleState {
    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "DELETING")]
    Deleting,

    #[serde(rename = "FAILED")]
    Failed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
