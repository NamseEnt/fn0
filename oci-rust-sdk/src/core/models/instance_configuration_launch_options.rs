use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Options for tuning the compatibility and performance of VM shapes. The values that you specify override any default values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchOptions {
    /// Emulation type for the boot volume. * {@code ISCSI} - ISCSI attached block storage device. * {@code SCSI} - Emulated SCSI disk. * {@code IDE} - Emulated IDE disk. * {@code VFIO} - Direct attached Virtual Function storage. This is the default option for local data volumes on platform images. * {@code PARAVIRTUALIZED} - Paravirtualized disk. This is the default for boot volumes and remote block storage volumes on platform images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_type: Option<InstanceConfigurationLaunchOptionsBootVolumeType>,

    /// Firmware used to boot VM. Select the option that matches your operating system. * {@code BIOS} - Boot VM using BIOS style firmware. This is compatible with both 32 bit and 64 bit operating systems that boot using MBR style bootloaders. * {@code UEFI_64} - Boot VM using UEFI style firmware compatible with 64 bit operating systems. This is the default for platform images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<InstanceConfigurationLaunchOptionsFirmware>,

    /// Emulation type for the physical network interface card (NIC). * {@code E1000} - Emulated Gigabit ethernet controller. Compatible with Linux e1000 network driver. * {@code VFIO} - Direct attached Virtual Function network controller. This is the networking type when you launch an instance using hardware-assisted (SR-IOV) networking. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_type: Option<InstanceConfigurationLaunchOptionsNetworkType>,

    /// Emulation type for volume. * {@code ISCSI} - ISCSI attached block storage device. * {@code SCSI} - Emulated SCSI disk. * {@code IDE} - Emulated IDE disk. * {@code VFIO} - Direct attached Virtual Function storage. This is the default option for local data volumes on platform images. * {@code PARAVIRTUALIZED} - Paravirtualized disk. This is the default for boot volumes and remote block storage volumes on platform images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_data_volume_type: Option<InstanceConfigurationLaunchOptionsRemoteDataVolumeType>,

    /// Deprecated. Instead use {@code isPvEncryptionInTransitEnabled} in {@link #instanceConfigurationLaunchInstanceDetails(InstanceConfigurationLaunchInstanceDetailsRequest) instanceConfigurationLaunchInstanceDetails}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// Whether to enable consistent volume naming feature. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_consistent_volume_naming_enabled: Option<bool>,
}

impl InstanceConfigurationLaunchOptions {
    /// Create a new InstanceConfigurationLaunchOptions
    pub fn new() -> Self {
        Self {
            boot_volume_type: None,

            firmware: None,

            network_type: None,

            remote_data_volume_type: None,

            is_pv_encryption_in_transit_enabled: None,

            is_consistent_volume_naming_enabled: None,
        }
    }

    /// Set boot_volume_type
    pub fn set_boot_volume_type(
        mut self,
        value: Option<InstanceConfigurationLaunchOptionsBootVolumeType>,
    ) -> Self {
        self.boot_volume_type = value;
        self
    }

    /// Set firmware
    pub fn set_firmware(
        mut self,
        value: Option<InstanceConfigurationLaunchOptionsFirmware>,
    ) -> Self {
        self.firmware = value;
        self
    }

    /// Set network_type
    pub fn set_network_type(
        mut self,
        value: Option<InstanceConfigurationLaunchOptionsNetworkType>,
    ) -> Self {
        self.network_type = value;
        self
    }

    /// Set remote_data_volume_type
    pub fn set_remote_data_volume_type(
        mut self,
        value: Option<InstanceConfigurationLaunchOptionsRemoteDataVolumeType>,
    ) -> Self {
        self.remote_data_volume_type = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set is_consistent_volume_naming_enabled
    pub fn set_is_consistent_volume_naming_enabled(mut self, value: Option<bool>) -> Self {
        self.is_consistent_volume_naming_enabled = value;
        self
    }

    /// Set boot_volume_type (unwraps Option)
    pub fn with_boot_volume_type(
        mut self,
        value: InstanceConfigurationLaunchOptionsBootVolumeType,
    ) -> Self {
        self.boot_volume_type = Some(value);
        self
    }

    /// Set firmware (unwraps Option)
    pub fn with_firmware(mut self, value: InstanceConfigurationLaunchOptionsFirmware) -> Self {
        self.firmware = Some(value);
        self
    }

    /// Set network_type (unwraps Option)
    pub fn with_network_type(
        mut self,
        value: InstanceConfigurationLaunchOptionsNetworkType,
    ) -> Self {
        self.network_type = Some(value);
        self
    }

    /// Set remote_data_volume_type (unwraps Option)
    pub fn with_remote_data_volume_type(
        mut self,
        value: InstanceConfigurationLaunchOptionsRemoteDataVolumeType,
    ) -> Self {
        self.remote_data_volume_type = Some(value);
        self
    }

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }

    /// Set is_consistent_volume_naming_enabled (unwraps Option)
    pub fn with_is_consistent_volume_naming_enabled(mut self, value: bool) -> Self {
        self.is_consistent_volume_naming_enabled = Some(value);
        self
    }
}

impl Default for InstanceConfigurationLaunchOptions {
    fn default() -> Self {
        Self::new()
    }
}
