use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageGroupGroupType {
    #[serde(rename = "GROUP")]
    Group,

    #[serde(rename = "ENVIRONMENT")]
    Environment,

    #[serde(rename = "CATEGORY")]
    Category,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
