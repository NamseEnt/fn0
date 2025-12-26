use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeAttachmentLifecycleState {
    #[serde(rename = "ATTACHING")]
    Attaching,

    #[serde(rename = "ATTACHED")]
    Attached,

    #[serde(rename = "DETACHING")]
    Detaching,

    #[serde(rename = "DETACHED")]
    Detached,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
