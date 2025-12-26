use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Options to handle timeout for pre-termination action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolPreTerminationActionHandleTimeoutDetails {
    /// Whether the block volume should be preserved after termination.
    pub preserve_block_volume_mode:
        InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBlockVolumeMode,

    /// Whether the boot volume should be preserved after termination.
    pub preserve_boot_volume_mode:
        InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBootVolumeMode,
}

/// Required fields for InstancePoolPreTerminationActionHandleTimeoutDetails
pub struct InstancePoolPreTerminationActionHandleTimeoutDetailsRequired {
    /// Whether the block volume should be preserved after termination.
    pub preserve_block_volume_mode:
        InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBlockVolumeMode,

    /// Whether the boot volume should be preserved after termination.
    pub preserve_boot_volume_mode:
        InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBootVolumeMode,
}

impl InstancePoolPreTerminationActionHandleTimeoutDetails {
    /// Create a new InstancePoolPreTerminationActionHandleTimeoutDetails with required fields
    pub fn new(required: InstancePoolPreTerminationActionHandleTimeoutDetailsRequired) -> Self {
        Self {
            preserve_block_volume_mode: required.preserve_block_volume_mode,

            preserve_boot_volume_mode: required.preserve_boot_volume_mode,
        }
    }

    /// Set preserve_block_volume_mode
    pub fn set_preserve_block_volume_mode(
        mut self,
        value: InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBlockVolumeMode,
    ) -> Self {
        self.preserve_block_volume_mode = value;
        self
    }

    /// Set preserve_boot_volume_mode
    pub fn set_preserve_boot_volume_mode(
        mut self,
        value: InstancePoolPreTerminationActionHandleTimeoutDetailsPreserveBootVolumeMode,
    ) -> Self {
        self.preserve_boot_volume_mode = value;
        self
    }
}
