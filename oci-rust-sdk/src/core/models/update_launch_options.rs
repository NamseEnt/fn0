use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Options for tuning the compatibility and performance of VM shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLaunchOptions {
    /// Emulation type for the boot volume. * {@code ISCSI} - ISCSI attached block storage device. * {@code PARAVIRTUALIZED} - Paravirtualized disk. This is the default for boot volumes and remote block storage volumes on platform images. <p> Before you change the boot volume attachment type, detach all block volumes and VNICs except for the boot volume and the primary VNIC. <p> If the instance is running when you change the boot volume attachment type, it will be rebooted. <p> *Note:** Some instances might not function properly if you change the boot volume attachment type. After the instance reboots and is running, connect to it. If the connection fails or the OS doesn't behave as expected, the changes are not supported. Revert the instance to the original boot volume attachment type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_type: Option<UpdateLaunchOptionsBootVolumeType>,

    /// Emulation type for the physical network interface card (NIC). * {@code VFIO} - Direct attached Virtual Function network controller. This is the networking type when you launch an instance using hardware-assisted (SR-IOV) networking. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers. <p> Before you change the networking type, detach all VNICs and block volumes except for the primary VNIC and the boot volume. <p> The image must have paravirtualized drivers installed. For more information, see [Editing an Instance](https://docs.oracle.com/iaas/Content/Compute/Tasks/resizinginstances.htm). <p> If the instance is running when you change the network type, it will be rebooted. <p> *Note:** Some instances might not function properly if you change the networking type. After the instance reboots and is running, connect to it. If the connection fails or the OS doesn't behave as expected, the changes are not supported. Revert the instance to the original networking type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_type: Option<UpdateLaunchOptionsNetworkType>,

    /// Whether to enable in-transit encryption for the volume's paravirtualized attachment. To enable in-transit encryption for block volumes and boot volumes, this field must be set to {@code true}. <p> Data in transit is transferred over an internal and highly secure network. If you have specific compliance requirements related to the encryption of the data while it is moving between the instance and the boot volume or the block volume, you can enable in-transit encryption. In-transit encryption is not enabled by default. <p> All boot volumes and block volumes are encrypted at rest. <p> For more information, see [Block Volume Encryption](https://docs.oracle.com/iaas/Content/Block/Concepts/overview.htm#Encrypti).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,
}

impl UpdateLaunchOptions {
    /// Create a new UpdateLaunchOptions
    pub fn new() -> Self {
        Self {
            boot_volume_type: None,

            network_type: None,

            is_pv_encryption_in_transit_enabled: None,
        }
    }

    /// Set boot_volume_type
    pub fn set_boot_volume_type(
        mut self,
        value: Option<UpdateLaunchOptionsBootVolumeType>,
    ) -> Self {
        self.boot_volume_type = value;
        self
    }

    /// Set network_type
    pub fn set_network_type(mut self, value: Option<UpdateLaunchOptionsNetworkType>) -> Self {
        self.network_type = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set boot_volume_type (unwraps Option)
    pub fn with_boot_volume_type(mut self, value: UpdateLaunchOptionsBootVolumeType) -> Self {
        self.boot_volume_type = Some(value);
        self
    }

    /// Set network_type (unwraps Option)
    pub fn with_network_type(mut self, value: UpdateLaunchOptionsNetworkType) -> Self {
        self.network_type = Some(value);
        self
    }

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }
}

impl Default for UpdateLaunchOptions {
    fn default() -> Self {
        Self::new()
    }
}
