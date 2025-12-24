use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootVolumeBackupLifecycleState {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "TERMINATED")]
    Terminated,

    #[serde(rename = "FAULTY")]
    Faulty,

    #[serde(rename = "REQUEST_RECEIVED")]
    RequestReceived,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
