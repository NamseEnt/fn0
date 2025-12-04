use serde::{Deserialize, Serialize};

/// Options for tuning the compatibility and performance of VM shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchOptions {
    /// Emulation type for the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_type: Option<BootVolumeType>,

    /// Firmware used to boot VM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<Firmware>,

    /// Emulation type for the physical network interface card (NIC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_type: Option<NetworkType>,

    /// Emulation type for volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_data_volume_type: Option<RemoteDataVolumeType>,

    /// Whether to enable in-transit encryption for paravirtualized attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// Whether to enable consistent volume naming feature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_consistent_volume_naming_enabled: Option<bool>,
}

/// Emulation type for boot volume and remote data volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BootVolumeType {
    /// ISCSI attached block storage device.
    Iscsi,
    /// Emulated SCSI disk.
    Scsi,
    /// Emulated IDE disk.
    Ide,
    /// Direct attached Virtual Function storage.
    Vfio,
    /// Paravirtualized disk.
    Paravirtualized,
}

/// Firmware used to boot VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Firmware {
    /// Boot VM using BIOS style firmware.
    Bios,
    /// Boot VM using UEFI style firmware compatible with 64 bit operating systems.
    #[serde(rename = "UEFI_64")]
    Uefi64,
}

/// Emulation type for the physical network interface card (NIC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkType {
    /// Emulated Gigabit ethernet controller.
    E1000,
    /// Direct attached Virtual Function network controller.
    Vfio,
    /// VM instances launch with paravirtualized devices using VirtIO drivers.
    Paravirtualized,
}

/// Emulation type for remote data volume (same as BootVolumeType).
pub type RemoteDataVolumeType = BootVolumeType;

impl InstanceConfigurationLaunchOptions {
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

    pub fn with_boot_volume_type(mut self, boot_volume_type: BootVolumeType) -> Self {
        self.boot_volume_type = Some(boot_volume_type);
        self
    }

    pub fn with_firmware(mut self, firmware: Firmware) -> Self {
        self.firmware = Some(firmware);
        self
    }

    pub fn with_network_type(mut self, network_type: NetworkType) -> Self {
        self.network_type = Some(network_type);
        self
    }

    pub fn with_remote_data_volume_type(mut self, volume_type: RemoteDataVolumeType) -> Self {
        self.remote_data_volume_type = Some(volume_type);
        self
    }
}

impl Default for InstanceConfigurationLaunchOptions {
    fn default() -> Self {
        Self::new()
    }
}
