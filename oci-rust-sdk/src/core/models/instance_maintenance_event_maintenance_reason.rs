use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventMaintenanceReason {
    #[serde(rename = "EVACUATION")]
    Evacuation,

    #[serde(rename = "ENVIRONMENTAL_FACTORS")]
    EnvironmentalFactors,

    #[serde(rename = "DECOMMISSION")]
    Decommission,

    #[serde(rename = "HARDWARE_REPLACEMENT")]
    HardwareReplacement,

    #[serde(rename = "FIRMWARE_UPDATE")]
    FirmwareUpdate,

    #[serde(rename = "SECURITY_UPDATE")]
    SecurityUpdate,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
