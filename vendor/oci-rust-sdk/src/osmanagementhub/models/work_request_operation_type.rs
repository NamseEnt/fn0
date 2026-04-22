use serde::{Deserialize, Serialize};

/// Possible operation types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkRequestOperationType {
    #[serde(rename = "INSTALL_PACKAGES")]
    InstallPackages,

    #[serde(rename = "REMOVE_PACKAGES")]
    RemovePackages,

    #[serde(rename = "UPDATE_PACKAGES")]
    UpdatePackages,

    #[serde(rename = "UPDATE_ALL_PACKAGES")]
    UpdateAllPackages,

    #[serde(rename = "UPDATE_SECURITY")]
    UpdateSecurity,

    #[serde(rename = "UPDATE_BUGFIX")]
    UpdateBugfix,

    #[serde(rename = "UPDATE_ENHANCEMENT")]
    UpdateEnhancement,

    #[serde(rename = "UPDATE_OTHER")]
    UpdateOther,

    #[serde(rename = "UPDATE_KSPLICE_KERNEL")]
    UpdateKspliceKernel,

    #[serde(rename = "UPDATE_KSPLICE_USERSPACE")]
    UpdateKspliceUserspace,

    #[serde(rename = "ENABLE_MODULE_STREAMS")]
    EnableModuleStreams,

    #[serde(rename = "DISABLE_MODULE_STREAMS")]
    DisableModuleStreams,

    #[serde(rename = "SWITCH_MODULE_STREAM")]
    SwitchModuleStream,

    #[serde(rename = "INSTALL_MODULE_PROFILES")]
    InstallModuleProfiles,

    #[serde(rename = "REMOVE_MODULE_PROFILES")]
    RemoveModuleProfiles,

    #[serde(rename = "SET_SOFTWARE_SOURCES")]
    SetSoftwareSources,

    #[serde(rename = "LIST_PACKAGES")]
    ListPackages,

    #[serde(rename = "SET_MANAGEMENT_STATION_CONFIG")]
    SetManagementStationConfig,

    #[serde(rename = "SYNC_MANAGEMENT_STATION_MIRROR")]
    SyncManagementStationMirror,

    #[serde(rename = "UPDATE_MANAGEMENT_STATION_SOFTWARE")]
    UpdateManagementStationSoftware,

    #[serde(rename = "UPDATE")]
    Update,

    #[serde(rename = "MODULE_ACTIONS")]
    ModuleActions,

    #[serde(rename = "LIFECYCLE_PROMOTION")]
    LifecyclePromotion,

    #[serde(rename = "CREATE_SOFTWARE_SOURCE")]
    CreateSoftwareSource,

    #[serde(rename = "UPDATE_SOFTWARE_SOURCE")]
    UpdateSoftwareSource,

    #[serde(rename = "IMPORT_CONTENT")]
    ImportContent,

    #[serde(rename = "SYNC_AGENT_CONFIG")]
    SyncAgentConfig,

    #[serde(rename = "INSTALL_WINDOWS_UPDATES")]
    InstallWindowsUpdates,

    #[serde(rename = "LIST_WINDOWS_UPDATE")]
    ListWindowsUpdate,

    #[serde(rename = "GET_WINDOWS_UPDATE_DETAILS")]
    GetWindowsUpdateDetails,

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

    #[serde(rename = "REMOVE_CONTENT")]
    RemoveContent,

    #[serde(rename = "UNREGISTER_MANAGED_INSTANCE")]
    UnregisterManagedInstance,

    #[serde(rename = "REBOOT")]
    Reboot,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
