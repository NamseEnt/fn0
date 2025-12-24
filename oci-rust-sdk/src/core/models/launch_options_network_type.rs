use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchOptionsNetworkType {
    #[serde(rename = "E1000")]
    E1000,

    #[serde(rename = "VFIO")]
    Vfio,

    #[serde(rename = "PARAVIRTUALIZED")]
    Paravirtualized,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
