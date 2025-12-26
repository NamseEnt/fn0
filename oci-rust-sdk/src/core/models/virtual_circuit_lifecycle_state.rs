use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualCircuitLifecycleState {
    #[serde(rename = "PENDING_PROVIDER")]
    PendingProvider,

    #[serde(rename = "VERIFYING")]
    Verifying,

    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "PROVISIONED")]
    Provisioned,

    #[serde(rename = "FAILED")]
    Failed,

    #[serde(rename = "INACTIVE")]
    Inactive,

    #[serde(rename = "TERMINATING")]
    Terminating,

    #[serde(rename = "TERMINATED")]
    Terminated,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
