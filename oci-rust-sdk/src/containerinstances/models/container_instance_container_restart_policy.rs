use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerInstanceContainerRestartPolicy {
    #[serde(rename = "ALWAYS")]
    Always,

    #[serde(rename = "NEVER")]
    Never,

    #[serde(rename = "ON_FAILURE")]
    OnFailure,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
