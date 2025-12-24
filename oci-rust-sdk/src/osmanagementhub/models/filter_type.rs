use serde::{Deserialize, Serialize};

/// Filter type for packages of a custom software source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    #[serde(rename = "INCLUDE")]
    Include,

    #[serde(rename = "EXCLUDE")]
    Exclude,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
