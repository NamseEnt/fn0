use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppCatalogListingResourceVersionAllowedActions {
    #[serde(rename = "SNAPSHOT")]
    Snapshot,

    #[serde(rename = "BOOT_VOLUME_DETACH")]
    BootVolumeDetach,

    #[serde(rename = "PRESERVE_BOOT_VOLUME")]
    PreserveBootVolume,

    #[serde(rename = "SERIAL_CONSOLE_ACCESS")]
    SerialConsoleAccess,

    #[serde(rename = "BOOT_RECOVERY")]
    BootRecovery,

    #[serde(rename = "BACKUP_BOOT_VOLUME")]
    BackupBootVolume,

    #[serde(rename = "CAPTURE_CONSOLE_HISTORY")]
    CaptureConsoleHistory,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
