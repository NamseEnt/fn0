use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details about management station actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestManagementStationDetails {
    /// Target version to update the management station software.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_station_version: Option<String>,

    /// The configuration of the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,

    /// Optional list for mirrors to sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_ids: Option<Vec<String>>,
}

impl WorkRequestManagementStationDetails {
    /// Create a new WorkRequestManagementStationDetails
    pub fn new() -> Self {
        Self {
            management_station_version: None,

            config: None,

            software_source_ids: None,
        }
    }

    /// Set management_station_version
    pub fn set_management_station_version(mut self, value: Option<String>) -> Self {
        self.management_station_version = value;
        self
    }

    /// Set config
    pub fn set_config(mut self, value: Option<String>) -> Self {
        self.config = value;
        self
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set management_station_version (unwraps Option)
    pub fn with_management_station_version(mut self, value: impl Into<String>) -> Self {
        self.management_station_version = Some(value.into());
        self
    }

    /// Set config (unwraps Option)
    pub fn with_config(mut self, value: impl Into<String>) -> Self {
        self.config = Some(value.into());
        self
    }

    /// Set software_source_ids (unwraps Option)
    pub fn with_software_source_ids(mut self, value: Vec<String>) -> Self {
        self.software_source_ids = Some(value);
        self
    }
}

impl Default for WorkRequestManagementStationDetails {
    fn default() -> Self {
        Self::new()
    }
}
