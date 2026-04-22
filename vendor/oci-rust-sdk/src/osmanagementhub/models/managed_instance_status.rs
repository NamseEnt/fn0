use serde::{Deserialize, Serialize};

/// Status of the managed instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagedInstanceStatus {
    #[serde(rename = "NORMAL")]
    Normal,

    #[serde(rename = "UNREACHABLE")]
    Unreachable,

    #[serde(rename = "ERROR")]
    Error,

    #[serde(rename = "WARNING")]
    Warning,

    #[serde(rename = "REGISTRATION_ERROR")]
    RegistrationError,

    #[serde(rename = "DELETING")]
    Deleting,

    #[serde(rename = "ONBOARDING")]
    Onboarding,

    #[serde(rename = "REBOOTING")]
    Rebooting,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
