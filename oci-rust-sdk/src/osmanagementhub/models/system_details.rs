use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information about the system architecture and operating system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDetails {
    /// Architecture type.
    pub architecture: ArchType,

    /// Operating system type.
    pub os_family: OsFamily,

    /// Name of the operating system.
    pub os_name: String,

    /// Version of the operating system.
    pub os_system_version: String,

    /// Version of the Ksplice effective kernel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ksplice_effective_kernel_version: Option<String>,

    /// Release of the kernel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_kernel_release: Option<String>,

    /// Version of the kernel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_kernel_version: Option<String>,
}

/// Required fields for SystemDetails
pub struct SystemDetailsRequired {
    /// Architecture type.
    pub architecture: ArchType,

    /// Operating system type.
    pub os_family: OsFamily,

    /// Name of the operating system.
    pub os_name: String,

    /// Version of the operating system.
    pub os_system_version: String,
}

impl SystemDetails {
    /// Create a new SystemDetails with required fields
    pub fn new(required: SystemDetailsRequired) -> Self {
        Self {
            architecture: required.architecture,

            os_family: required.os_family,

            os_name: required.os_name,

            os_system_version: required.os_system_version,

            ksplice_effective_kernel_version: None,

            os_kernel_release: None,

            os_kernel_version: None,
        }
    }

    /// Set architecture
    pub fn set_architecture(mut self, value: ArchType) -> Self {
        self.architecture = value;
        self
    }

    /// Set ksplice_effective_kernel_version
    pub fn set_ksplice_effective_kernel_version(mut self, value: Option<String>) -> Self {
        self.ksplice_effective_kernel_version = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set os_name
    pub fn set_os_name(mut self, value: String) -> Self {
        self.os_name = value;
        self
    }

    /// Set os_kernel_release
    pub fn set_os_kernel_release(mut self, value: Option<String>) -> Self {
        self.os_kernel_release = value;
        self
    }

    /// Set os_kernel_version
    pub fn set_os_kernel_version(mut self, value: Option<String>) -> Self {
        self.os_kernel_version = value;
        self
    }

    /// Set os_system_version
    pub fn set_os_system_version(mut self, value: String) -> Self {
        self.os_system_version = value;
        self
    }

    /// Set ksplice_effective_kernel_version (unwraps Option)
    pub fn with_ksplice_effective_kernel_version(mut self, value: impl Into<String>) -> Self {
        self.ksplice_effective_kernel_version = Some(value.into());
        self
    }

    /// Set os_kernel_release (unwraps Option)
    pub fn with_os_kernel_release(mut self, value: impl Into<String>) -> Self {
        self.os_kernel_release = Some(value.into());
        self
    }

    /// Set os_kernel_version (unwraps Option)
    pub fn with_os_kernel_version(mut self, value: impl Into<String>) -> Self {
        self.os_kernel_version = Some(value.into());
        self
    }
}
