use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventMaintenanceCategory {
    #[serde(rename = "EMERGENCY")]
    Emergency,

    #[serde(rename = "MANDATORY")]
    Mandatory,

    #[serde(rename = "FLEXIBLE")]
    Flexible,

    #[serde(rename = "OPTIONAL")]
    Optional,

    #[serde(rename = "NOTIFICATION")]
    Notification,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
