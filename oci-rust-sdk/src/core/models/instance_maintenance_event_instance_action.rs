use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventInstanceAction {
    #[serde(rename = "REBOOT_MIGRATION")]
    RebootMigration,

    #[serde(rename = "TERMINATE")]
    Terminate,

    #[serde(rename = "STOP")]
    Stop,

    #[serde(rename = "NONE")]
    None,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
