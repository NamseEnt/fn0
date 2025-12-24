use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstancePoolSummaryLifecycleState {
    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "SCALING")]
    Scaling,

    #[serde(rename = "STARTING")]
    Starting,

    #[serde(rename = "STOPPING")]
    Stopping,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "STOPPED")]
    Stopped,

    #[serde(rename = "TERMINATED")]
    Terminated,

    #[serde(rename = "RUNNING")]
    Running,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
