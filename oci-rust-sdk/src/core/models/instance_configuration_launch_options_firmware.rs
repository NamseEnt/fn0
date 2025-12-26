use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceConfigurationLaunchOptionsFirmware {
    #[serde(rename = "BIOS")]
    Bios,

    #[serde(rename = "UEFI_64")]
    Uefi64,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
