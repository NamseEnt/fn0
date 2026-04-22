use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchInstanceShapeConfigDetailsResourceManagement {
    #[serde(rename = "DYNAMIC")]
    Dynamic,

    #[serde(rename = "STATIC")]
    Static,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
