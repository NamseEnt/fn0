use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareSourceEventDataOperationType {
    #[serde(rename = "ENABLE_MODULE_STREAMS")]
    EnableModuleStreams,

    #[serde(rename = "DISABLE_MODULE_STREAMS")]
    DisableModuleStreams,

    #[serde(rename = "SWITCH_MODULE_STREAM")]
    SwitchModuleStream,

    #[serde(rename = "INSTALL_MODULE_PROFILE")]
    InstallModuleProfile,

    #[serde(rename = "REMOVE_MODULE_PROFILES")]
    RemoveModuleProfiles,

    #[serde(rename = "SET_SOFTWARE_SOURCES")]
    SetSoftwareSources,

    #[serde(rename = "UPDATE_SOFTWARE_SOURCE")]
    UpdateSoftwareSource,

    #[serde(rename = "CREATE_SOFTWARE_SOURCE")]
    CreateSoftwareSource,

    #[serde(rename = "LIFECYCLE_PROMOTION")]
    LifecyclePromotion,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
