use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The preference object specified by customer. Contains customerDesiredFirmwareBundleId, fabricRecycleLevel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFabricPreferencesDescriptor {
    /// The desired firmware bundle id on the GPU memory fabric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_desired_firmware_bundle_id: Option<String>,

    /// The recycle level of GPU memory fabric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fabric_recycle_level: Option<MemoryFabricPreferencesDescriptorFabricRecycleLevel>,
}

impl MemoryFabricPreferencesDescriptor {
    /// Create a new MemoryFabricPreferencesDescriptor
    pub fn new() -> Self {
        Self {
            customer_desired_firmware_bundle_id: None,

            fabric_recycle_level: None,
        }
    }

    /// Set customer_desired_firmware_bundle_id
    pub fn set_customer_desired_firmware_bundle_id(mut self, value: Option<String>) -> Self {
        self.customer_desired_firmware_bundle_id = value;
        self
    }

    /// Set fabric_recycle_level
    pub fn set_fabric_recycle_level(
        mut self,
        value: Option<MemoryFabricPreferencesDescriptorFabricRecycleLevel>,
    ) -> Self {
        self.fabric_recycle_level = value;
        self
    }

    /// Set customer_desired_firmware_bundle_id (unwraps Option)
    pub fn with_customer_desired_firmware_bundle_id(mut self, value: impl Into<String>) -> Self {
        self.customer_desired_firmware_bundle_id = Some(value.into());
        self
    }

    /// Set fabric_recycle_level (unwraps Option)
    pub fn with_fabric_recycle_level(
        mut self,
        value: MemoryFabricPreferencesDescriptorFabricRecycleLevel,
    ) -> Self {
        self.fabric_recycle_level = Some(value);
        self
    }
}

impl Default for MemoryFabricPreferencesDescriptor {
    fn default() -> Self {
        Self::new()
    }
}
