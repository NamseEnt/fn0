use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByoipRangeLifecycleDetails {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "VALIDATING")]
    Validating,

    #[serde(rename = "PROVISIONED")]
    Provisioned,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "FAILED")]
    Failed,

    #[serde(rename = "DELETING")]
    Deleting,

    #[serde(rename = "DELETED")]
    Deleted,

    #[serde(rename = "ADVERTISING")]
    Advertising,

    #[serde(rename = "WITHDRAWING")]
    Withdrawing,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
