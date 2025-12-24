use serde::{Deserialize, Serialize};

/// Indicates requirements for installing updates on a managed instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationRequirements {
    #[serde(rename = "EULA_ACCEPTANCE_REQUIRED")]
    EulaAcceptanceRequired,

    #[serde(rename = "SOFTWARE_MEDIA_REQUIRED")]
    SoftwareMediaRequired,

    #[serde(rename = "USER_INTERACTION_REQUIRED")]
    UserInteractionRequired,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
