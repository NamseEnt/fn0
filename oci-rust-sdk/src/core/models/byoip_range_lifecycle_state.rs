use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByoipRangeLifecycleState {
    #[serde(rename = "INACTIVE")]
    Inactive,

    #[serde(rename = "UPDATING")]
    Updating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DELETING")]
    Deleting,

    #[serde(rename = "DELETED")]
    Deleted,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
