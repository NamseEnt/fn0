use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareUpdateEventDataOperationType {
    #[serde(rename = "UPDATE_ALL_PACKAGES")]
    UpdateAllPackages,

    #[serde(rename = "INSTALL_PACKAGES")]
    InstallPackages,

    #[serde(rename = "REMOVE_PACKAGES")]
    RemovePackages,

    #[serde(rename = "UPDATE_PACKAGES")]
    UpdatePackages,

    #[serde(rename = "UPDATE_SECURITY")]
    UpdateSecurity,

    #[serde(rename = "UPDATE_BUGFIX")]
    UpdateBugfix,

    #[serde(rename = "UPDATE_ENHANCEMENT")]
    UpdateEnhancement,

    #[serde(rename = "UPDATE_OTHER")]
    UpdateOther,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
