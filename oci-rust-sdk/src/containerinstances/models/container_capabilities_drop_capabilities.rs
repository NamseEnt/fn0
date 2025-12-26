use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for ContainerCapabilitiesDropCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapChown => write!(f, "CAP_CHOWN"),

            Self::CapDacOverride => write!(f, "CAP_DAC_OVERRIDE"),

            Self::CapFsetid => write!(f, "CAP_FSETID"),

            Self::CapFowner => write!(f, "CAP_FOWNER"),

            Self::CapMknod => write!(f, "CAP_MKNOD"),

            Self::CapNetRaw => write!(f, "CAP_NET_RAW"),

            Self::CapSetgid => write!(f, "CAP_SETGID"),

            Self::CapSetuid => write!(f, "CAP_SETUID"),

            Self::CapSetfcap => write!(f, "CAP_SETFCAP"),

            Self::CapSetpcap => write!(f, "CAP_SETPCAP"),

            Self::CapNetBindService => write!(f, "CAP_NET_BIND_SERVICE"),

            Self::CapSysChroot => write!(f, "CAP_SYS_CHROOT"),

            Self::CapKill => write!(f, "CAP_KILL"),

            Self::CapAuditWrite => write!(f, "CAP_AUDIT_WRITE"),

            Self::All => write!(f, "ALL"),

            Self::UnknownValue => write!(f, "UNKNOWN"),
        }
    }
}
