use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicIpLifecycleState {
    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "ASSIGNING")]
    Assigning,

    #[serde(rename = "ASSIGNED")]
    Assigned,

    #[serde(rename = "UNASSIGNING")]
    Unassigning,

    #[serde(rename = "UNASSIGNED")]
    Unassigned,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "TERMINATED")]
    Terminated,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
