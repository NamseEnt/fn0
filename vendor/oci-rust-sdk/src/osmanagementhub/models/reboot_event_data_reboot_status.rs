use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebootEventDataRebootStatus {
    #[serde(rename = "REBOOT_STARTED")]
    RebootStarted,

    #[serde(rename = "REBOOT_SUCCEEDED")]
    RebootSucceeded,

    #[serde(rename = "REBOOT_FAILED")]
    RebootFailed,

    #[serde(rename = "REBOOT_SUCCEEDED_AFTER_TIMEOUT")]
    RebootSucceededAfterTimeout,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
