use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeCapacityTopologyLifecycleState {
    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "UPDATING")]
    Updating,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "DELETING")]
    Deleting,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
