use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceMaintenanceEventLifecycleState {
    #[serde(rename = "SCHEDULED")]
    Scheduled,

    #[serde(rename = "STARTED")]
    Started,

    #[serde(rename = "PROCESSING")]
    Processing,

    #[serde(rename = "SUCCEEDED")]
    Succeeded,

    #[serde(rename = "FAILED")]
    Failed,

    #[serde(rename = "CANCELED")]
    Canceled,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
