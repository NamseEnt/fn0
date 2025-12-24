use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerCapabilitiesDropCapabilities {
    #[serde(rename = "CAP_CHOWN")]
    CapChown,

    #[serde(rename = "CAP_DAC_OVERRIDE")]
    CapDacOverride,

    #[serde(rename = "CAP_FSETID")]
    CapFsetid,

    #[serde(rename = "CAP_FOWNER")]
    CapFowner,

    #[serde(rename = "CAP_MKNOD")]
    CapMknod,

    #[serde(rename = "CAP_NET_RAW")]
    CapNetRaw,

    #[serde(rename = "CAP_SETGID")]
    CapSetgid,

    #[serde(rename = "CAP_SETUID")]
    CapSetuid,

    #[serde(rename = "CAP_SETFCAP")]
    CapSetfcap,

    #[serde(rename = "CAP_SETPCAP")]
    CapSetpcap,

    #[serde(rename = "CAP_NET_BIND_SERVICE")]
    CapNetBindService,

    #[serde(rename = "CAP_SYS_CHROOT")]
    CapSysChroot,

    #[serde(rename = "CAP_KILL")]
    CapKill,

    #[serde(rename = "CAP_AUDIT_WRITE")]
    CapAuditWrite,

    #[serde(rename = "ALL")]
    All,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
