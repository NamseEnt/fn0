use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagementStationEventDataOperationType {
    #[serde(rename = "SET_MANAGEMENT_STATION_CONFIG")]
    SetManagementStationConfig,

    #[serde(rename = "SYNC_MANAGEMENT_STATION_MIRROR")]
    SyncManagementStationMirror,

    #[serde(rename = "UPDATE_MANAGEMENT_STATION_SOFTWARE")]
    UpdateManagementStationSoftware,

    #[serde(rename = "SET_MANAGEMENT_STATION_HEALTH_STATE")]
    SetManagementStationHealthState,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
