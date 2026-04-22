use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateLaunchOptionsNetworkType {
    #[serde(rename = "VFIO")]
    Vfio,

    #[serde(rename = "PARAVIRTUALIZED")]
    Paravirtualized,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
