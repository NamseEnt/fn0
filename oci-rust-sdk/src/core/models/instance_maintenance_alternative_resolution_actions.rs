use serde::{Deserialize, Serialize};

/// These are alternative actions to the requested instanceAction that can be taken to resolve the Maintenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceAlternativeResolutionActions {
    #[serde(rename = "REBOOT_MIGRATION")]
    RebootMigration,

    #[serde(rename = "TERMINATE")]
    Terminate,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
