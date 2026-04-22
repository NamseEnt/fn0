use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Host group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostGroupConfiguration {
    /// Either the platform name or compute shape that the configuration is targeting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// The OCID for firmware bundle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_bundle_id: Option<String>,

    /// Preferred recycle level for hosts associated with the reservation config. * {@code SKIP_RECYCLE} - Skips host wipe. * {@code FULL_RECYCLE} - Does not skip host wipe. This is the default behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recycle_level: Option<HostGroupConfigurationRecycleLevel>,
}

impl HostGroupConfiguration {
    /// Create a new HostGroupConfiguration
    pub fn new() -> Self {
        Self {
            target: None,

            firmware_bundle_id: None,

            recycle_level: None,
        }
    }

    /// Set target
    pub fn set_target(mut self, value: Option<String>) -> Self {
        self.target = value;
        self
    }

    /// Set firmware_bundle_id
    pub fn set_firmware_bundle_id(mut self, value: Option<String>) -> Self {
        self.firmware_bundle_id = value;
        self
    }

    /// Set recycle_level
    pub fn set_recycle_level(mut self, value: Option<HostGroupConfigurationRecycleLevel>) -> Self {
        self.recycle_level = value;
        self
    }

    /// Set target (unwraps Option)
    pub fn with_target(mut self, value: impl Into<String>) -> Self {
        self.target = Some(value.into());
        self
    }

    /// Set firmware_bundle_id (unwraps Option)
    pub fn with_firmware_bundle_id(mut self, value: impl Into<String>) -> Self {
        self.firmware_bundle_id = Some(value.into());
        self
    }

    /// Set recycle_level (unwraps Option)
    pub fn with_recycle_level(mut self, value: HostGroupConfigurationRecycleLevel) -> Self {
        self.recycle_level = Some(value);
        self
    }
}

impl Default for HostGroupConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
