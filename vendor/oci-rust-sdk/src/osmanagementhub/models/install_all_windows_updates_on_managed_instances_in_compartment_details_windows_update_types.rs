use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallAllWindowsUpdatesOnManagedInstancesInCompartmentDetailsWindowsUpdateTypes {
    #[serde(rename = "SECURITY")]
    Security,

    #[serde(rename = "BUGFIX")]
    Bugfix,

    #[serde(rename = "ENHANCEMENT")]
    Enhancement,

    #[serde(rename = "OTHER")]
    Other,

    #[serde(rename = "ALL")]
    All,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
