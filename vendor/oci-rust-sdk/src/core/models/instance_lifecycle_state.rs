use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceLifecycleState {
    #[serde(rename = "MOVING")]
    Moving,

    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "RUNNING")]
    Running,

    #[serde(rename = "STARTING")]
    Starting,

    #[serde(rename = "STOPPING")]
    Stopping,

    #[serde(rename = "STOPPED")]
    Stopped,

    #[serde(rename = "CREATING_IMAGE")]
    CreatingImage,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "TERMINATED")]
    Terminated,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
