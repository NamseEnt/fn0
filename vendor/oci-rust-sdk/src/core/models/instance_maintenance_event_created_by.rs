use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventCreatedBy {
    #[serde(rename = "CUSTOMER")]
    Customer,

    #[serde(rename = "SYSTEM")]
    System,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
