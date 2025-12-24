use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootVolumeReplicaLifecycleState {
    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "ACTIVATING")]
    Activating,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "TERMINATED")]
    Terminated,

    #[serde(rename = "FAULTY")]
    Faulty,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
