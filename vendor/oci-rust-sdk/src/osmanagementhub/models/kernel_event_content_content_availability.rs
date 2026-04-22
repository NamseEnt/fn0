use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEventContentContentAvailability {
    #[serde(rename = "NOT_AVAILABLE")]
    NotAvailable,

    #[serde(rename = "AVAILABLE_ON_INSTANCE")]
    AvailableOnInstance,

    #[serde(rename = "AVAILABLE_ON_SERVICE")]
    AvailableOnService,

    #[serde(rename = "AVAILABLE_ON_INSTANCE_AND_SERVICE")]
    AvailableOnInstanceAndService,

    #[serde(rename = "AVAILABLE_ON_INSTANCE_UPLOAD_IN_PROGRESS")]
    AvailableOnInstanceUploadInProgress,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
