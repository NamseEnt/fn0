use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureFilterFilterType {
    #[serde(rename = "VTAP")]
    Vtap,

    #[serde(rename = "FLOWLOG")]
    Flowlog,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
