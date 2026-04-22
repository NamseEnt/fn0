use serde::{Deserialize, Serialize};

/// Availability status of a software source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "SELECTED")]
    Selected,

    #[serde(rename = "RESTRICTED")]
    Restricted,

    #[serde(rename = "UNAVAILABLE")]
    Unavailable,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
