use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of other management stations that are behind the same load balancer within a high availability configuration. Stations are identified as peers if they have the same hostname and compartment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerManagementStation {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// User-friendly name for the management station.
    pub display_name: String,
}

/// Required fields for PeerManagementStation
pub struct PeerManagementStationRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station.
    pub id: String,

    /// User-friendly name for the management station.
    pub display_name: String,
}

impl PeerManagementStation {
    /// Create a new PeerManagementStation with required fields
    pub fn new(required: PeerManagementStationRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }
}
