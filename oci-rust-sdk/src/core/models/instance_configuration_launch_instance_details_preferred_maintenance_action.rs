use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceConfigurationLaunchInstanceDetailsPreferredMaintenanceAction {
    #[serde(rename = "LIVE_MIGRATE")]
    LiveMigrate,

    #[serde(rename = "REBOOT")]
    Reboot,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
