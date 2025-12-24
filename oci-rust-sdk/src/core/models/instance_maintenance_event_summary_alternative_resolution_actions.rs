use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventSummaryAlternativeResolutionActions {
    #[serde(rename = "REBOOT_MIGRATION")]
    RebootMigration,

    #[serde(rename = "TERMINATE")]
    Terminate,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
