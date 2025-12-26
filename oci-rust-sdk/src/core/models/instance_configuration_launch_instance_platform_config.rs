use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The platform configuration requested for the instance. <p> If you provide the parameter, the instance is created with the platform configuration that you specify. For any values that you omit, the instance uses the default configuration values for the {@code shape} that you specify. If you don't provide the parameter, the default values for the {@code shape} are used. <p> Each shape only supports certain configurable values. If the values that you provide are not valid for the specified {@code shape}, an error is returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstancePlatformConfig {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether Secure Boot is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_secure_boot_enabled: Option<bool>,

    /// Whether the Trusted Platform Module (TPM) is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trusted_platform_module_enabled: Option<bool>,

    /// Whether the Measured Boot feature is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_measured_boot_enabled: Option<bool>,

    /// Whether the instance is a confidential instance. If this value is {@code true}, the instance is a confidential instance. The default value is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,
}

/// Required fields for InstanceConfigurationLaunchInstancePlatformConfig
pub struct InstanceConfigurationLaunchInstancePlatformConfigRequired {
    pub r#type: String,
}

impl InstanceConfigurationLaunchInstancePlatformConfig {
    /// Create a new InstanceConfigurationLaunchInstancePlatformConfig with required fields
    pub fn new(required: InstanceConfigurationLaunchInstancePlatformConfigRequired) -> Self {
        Self {
            r#type: required.r#type,

            is_secure_boot_enabled: None,

            is_trusted_platform_module_enabled: None,

            is_measured_boot_enabled: None,

            is_memory_encryption_enabled: None,
        }
    }

    /// Set is_secure_boot_enabled
    pub fn set_is_secure_boot_enabled(mut self, value: Option<bool>) -> Self {
        self.is_secure_boot_enabled = value;
        self
    }

    /// Set is_trusted_platform_module_enabled
    pub fn set_is_trusted_platform_module_enabled(mut self, value: Option<bool>) -> Self {
        self.is_trusted_platform_module_enabled = value;
        self
    }

    /// Set is_measured_boot_enabled
    pub fn set_is_measured_boot_enabled(mut self, value: Option<bool>) -> Self {
        self.is_measured_boot_enabled = value;
        self
    }

    /// Set is_memory_encryption_enabled
    pub fn set_is_memory_encryption_enabled(mut self, value: Option<bool>) -> Self {
        self.is_memory_encryption_enabled = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set is_secure_boot_enabled (unwraps Option)
    pub fn with_is_secure_boot_enabled(mut self, value: bool) -> Self {
        self.is_secure_boot_enabled = Some(value);
        self
    }

    /// Set is_trusted_platform_module_enabled (unwraps Option)
    pub fn with_is_trusted_platform_module_enabled(mut self, value: bool) -> Self {
        self.is_trusted_platform_module_enabled = Some(value);
        self
    }

    /// Set is_measured_boot_enabled (unwraps Option)
    pub fn with_is_measured_boot_enabled(mut self, value: bool) -> Self {
        self.is_measured_boot_enabled = Some(value);
        self
    }

    /// Set is_memory_encryption_enabled (unwraps Option)
    pub fn with_is_memory_encryption_enabled(mut self, value: bool) -> Self {
        self.is_memory_encryption_enabled = Some(value);
        self
    }
}
