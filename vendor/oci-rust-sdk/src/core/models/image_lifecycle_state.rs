use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageLifecycleState {
    #[serde(rename = "PROVISIONING")]
    Provisioning,

    #[serde(rename = "IMPORTING")]
    Importing,

    #[serde(rename = "AVAILABLE")]
    Available,

    #[serde(rename = "EXPORTING")]
    Exporting,

    #[serde(rename = "DISABLED")]
    Disabled,

    #[serde(rename = "DELETED")]
    Deleted,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
