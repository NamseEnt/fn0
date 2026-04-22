use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByoasnLifecycleState {
    #[serde(rename = "UPDATING")]
    Updating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "FAILED")]
    Failed,

    #[serde(rename = "CREATING")]
    Creating,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
