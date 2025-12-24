use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the management station details used to configure a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStationDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as primary management station.
    pub primary_management_station_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as secondary managment station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_management_station_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for ManagementStationDetails
pub struct ManagementStationDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station for the instance to use as primary management station.
    pub primary_management_station_id: String,
}

impl ManagementStationDetails {
    /// Create a new ManagementStationDetails with required fields
    pub fn new(required: ManagementStationDetailsRequired) -> Self {
        Self {
            primary_management_station_id: required.primary_management_station_id,

            secondary_management_station_id: None,

            work_request_details: None,
        }
    }

    /// Set primary_management_station_id
    pub fn set_primary_management_station_id(mut self, value: String) -> Self {
        self.primary_management_station_id = value;
        self
    }

    /// Set secondary_management_station_id
    pub fn set_secondary_management_station_id(mut self, value: Option<String>) -> Self {
        self.secondary_management_station_id = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set secondary_management_station_id (unwraps Option)
    pub fn with_secondary_management_station_id(mut self, value: impl Into<String>) -> Self {
        self.secondary_management_station_id = Some(value.into());
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
