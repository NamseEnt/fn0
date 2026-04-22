use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapacityReportShapeAvailabilityAvailabilityStatus {
    #[serde(rename = "OUT_OF_HOST_CAPACITY")]
    OutOfHostCapacity,

    #[serde(rename = "HARDWARE_NOT_SUPPORTED")]
    HardwareNotSupported,

    #[serde(rename = "AVAILABLE")]
    Available,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
