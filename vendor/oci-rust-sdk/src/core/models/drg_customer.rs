use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The list of IPSEC / FC / RPC info for a given DRG
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgCustomer {
    /// OCID of the DRG
    pub drg_id: String,

    /// A List of the RPC connections on this DRG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_peering_connections: Option<Vec<DrgCustomerResource>>,

    /// A List of the VCs on this DRG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_circuits: Option<Vec<DrgCustomerResource>>,

    /// A List of the IPSec connections on this DRG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec_connections: Option<Vec<DrgCustomerResource>>,
}

/// Required fields for DrgCustomer
pub struct DrgCustomerRequired {
    /// OCID of the DRG
    pub drg_id: String,
}

impl DrgCustomer {
    /// Create a new DrgCustomer with required fields
    pub fn new(required: DrgCustomerRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            remote_peering_connections: None,

            virtual_circuits: None,

            ipsec_connections: None,
        }
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
        self
    }

    /// Set remote_peering_connections
    pub fn set_remote_peering_connections(
        mut self,
        value: Option<Vec<DrgCustomerResource>>,
    ) -> Self {
        self.remote_peering_connections = value;
        self
    }

    /// Set virtual_circuits
    pub fn set_virtual_circuits(mut self, value: Option<Vec<DrgCustomerResource>>) -> Self {
        self.virtual_circuits = value;
        self
    }

    /// Set ipsec_connections
    pub fn set_ipsec_connections(mut self, value: Option<Vec<DrgCustomerResource>>) -> Self {
        self.ipsec_connections = value;
        self
    }

    /// Set remote_peering_connections (unwraps Option)
    pub fn with_remote_peering_connections(mut self, value: Vec<DrgCustomerResource>) -> Self {
        self.remote_peering_connections = Some(value);
        self
    }

    /// Set virtual_circuits (unwraps Option)
    pub fn with_virtual_circuits(mut self, value: Vec<DrgCustomerResource>) -> Self {
        self.virtual_circuits = Some(value);
        self
    }

    /// Set ipsec_connections (unwraps Option)
    pub fn with_ipsec_connections(mut self, value: Vec<DrgCustomerResource>) -> Self {
        self.ipsec_connections = Some(value);
        self
    }
}
