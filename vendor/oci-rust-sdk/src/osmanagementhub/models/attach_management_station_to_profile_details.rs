use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach a management station to a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachManagementStationToProfileDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station that the instance will be associated with.
    pub management_station_id: String,
}

/// Required fields for AttachManagementStationToProfileDetails
pub struct AttachManagementStationToProfileDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station that the instance will be associated with.
    pub management_station_id: String,
}

impl AttachManagementStationToProfileDetails {
    /// Create a new AttachManagementStationToProfileDetails with required fields
    pub fn new(required: AttachManagementStationToProfileDetailsRequired) -> Self {
        Self {
            management_station_id: required.management_station_id,
        }
    }

    /// Set management_station_id
    pub fn set_management_station_id(mut self, value: String) -> Self {
        self.management_station_id = value;
        self
    }
}
