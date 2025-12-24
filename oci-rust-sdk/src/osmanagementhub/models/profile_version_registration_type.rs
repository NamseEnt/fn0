use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileVersionRegistrationType {
    #[serde(rename = "OCI_LINUX")]
    OciLinux,

    #[serde(rename = "NON_OCI_LINUX")]
    NonOciLinux,

    #[serde(rename = "OCI_WINDOWS")]
    OciWindows,

    #[serde(rename = "AUTONOMOUS_LINUX")]
    AutonomousLinux,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
