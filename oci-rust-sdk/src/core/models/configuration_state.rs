use serde::{Deserialize, Serialize};

/// The current state of the host configuration. The Host is either CONFORMANT - current state matches the desired configuration NON_CONFORMANT - current state does not match the desired configuration PRE_APPLYING, APPLYING, CHECKING - transitional states UNKNOWN - current state is unknown
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationState {
    #[serde(rename = "CONFORMANT")]
    Conformant,

    #[serde(rename = "NON_CONFORMANT")]
    NonConformant,

    #[serde(rename = "CHECKING")]
    Checking,

    #[serde(rename = "PRE_APPLYING")]
    PreApplying,

    #[serde(rename = "APPLYING")]
    Applying,

    #[serde(rename = "UNKNOWN")]
    Unknown,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
