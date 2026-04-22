use serde::{Deserialize, Serialize};

/// Type of operation the scheduled job is performing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationTypes {
    #[serde(rename = "INSTALL_PACKAGES")]
    InstallPackages,

    #[serde(rename = "UPDATE_PACKAGES")]
    UpdatePackages,

    #[serde(rename = "REMOVE_PACKAGES")]
    RemovePackages,

    #[serde(rename = "UPDATE_ALL")]
    UpdateAll,

    #[serde(rename = "UPDATE_SECURITY")]
    UpdateSecurity,

    #[serde(rename = "UPDATE_BUGFIX")]
    UpdateBugfix,

    #[serde(rename = "UPDATE_ENHANCEMENT")]
    UpdateEnhancement,

    #[serde(rename = "UPDATE_OTHER")]
    UpdateOther,

    #[serde(rename = "UPDATE_KSPLICE_USERSPACE")]
    UpdateKspliceUserspace,

    #[serde(rename = "UPDATE_KSPLICE_KERNEL")]
    UpdateKspliceKernel,

    #[serde(rename = "MANAGE_MODULE_STREAMS")]
    ManageModuleStreams,

    #[serde(rename = "SWITCH_MODULE_STREAM")]
    SwitchModuleStream,

    #[serde(rename = "ATTACH_SOFTWARE_SOURCES")]
    AttachSoftwareSources,

    #[serde(rename = "DETACH_SOFTWARE_SOURCES")]
    DetachSoftwareSources,

    #[serde(rename = "SYNC_MANAGEMENT_STATION_MIRROR")]
    SyncManagementStationMirror,

    #[serde(rename = "PROMOTE_LIFECYCLE")]
    PromoteLifecycle,

    #[serde(rename = "INSTALL_WINDOWS_UPDATES")]
    InstallWindowsUpdates,

    #[serde(rename = "INSTALL_ALL_WINDOWS_UPDATES")]
    InstallAllWindowsUpdates,

    #[serde(rename = "INSTALL_SECURITY_WINDOWS_UPDATES")]
    InstallSecurityWindowsUpdates,

    #[serde(rename = "INSTALL_BUGFIX_WINDOWS_UPDATES")]
    InstallBugfixWindowsUpdates,

    #[serde(rename = "INSTALL_ENHANCEMENT_WINDOWS_UPDATES")]
    InstallEnhancementWindowsUpdates,

    #[serde(rename = "INSTALL_OTHER_WINDOWS_UPDATES")]
    InstallOtherWindowsUpdates,

    #[serde(rename = "REBOOT")]
    Reboot,

    #[serde(rename = "RERUN_WORK_REQUEST")]
    RerunWorkRequest,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
