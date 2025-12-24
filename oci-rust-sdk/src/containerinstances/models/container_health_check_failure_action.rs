use serde::{Deserialize, Serialize};

/// The failure action will be taken on behalf of container instance status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerHealthCheckFailureAction {
    #[serde(rename = "KILL")]
    Kill,

    #[serde(rename = "NONE")]
    None,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
