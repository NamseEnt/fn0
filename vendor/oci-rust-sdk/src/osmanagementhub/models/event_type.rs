use serde::{Deserialize, Serialize};

/// Possible event types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "KERNEL_OOPS")]
    KernelOops,

    #[serde(rename = "KERNEL_CRASH")]
    KernelCrash,

    #[serde(rename = "EXPLOIT_ATTEMPT")]
    ExploitAttempt,

    #[serde(rename = "SOFTWARE_UPDATE")]
    SoftwareUpdate,

    #[serde(rename = "KSPLICE_UPDATE")]
    KspliceUpdate,

    #[serde(rename = "SOFTWARE_SOURCE")]
    SoftwareSource,

    #[serde(rename = "AGENT")]
    Agent,

    #[serde(rename = "MANAGEMENT_STATION")]
    ManagementStation,

    #[serde(rename = "SYSADMIN")]
    Sysadmin,

    #[serde(rename = "REBOOT")]
    Reboot,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
