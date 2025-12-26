use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The default DRG route table for this DRG. Each network type has a default DRG route table. <p> You can update a network type to use a different DRG route table, but each network type must have a default DRG route table. You cannot delete a default DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultDrgRouteTables {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the default DRG route table to be assigned to DRG attachments of type VCN on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the default DRG route table assigned to DRG attachments of type IPSEC_TUNNEL on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec_tunnel: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the default DRG route table to be assigned to DRG attachments of type VIRTUAL_CIRCUIT on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_circuit: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the default DRG route table to be assigned to DRG attachments of type REMOTE_PEERING_CONNECTION on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_peering_connection: Option<String>,
}

impl DefaultDrgRouteTables {
    /// Create a new DefaultDrgRouteTables
    pub fn new() -> Self {
        Self {
            vcn: None,

            ipsec_tunnel: None,

            virtual_circuit: None,

            remote_peering_connection: None,
        }
    }

    /// Set vcn
    pub fn set_vcn(mut self, value: Option<String>) -> Self {
        self.vcn = value;
        self
    }

    /// Set ipsec_tunnel
    pub fn set_ipsec_tunnel(mut self, value: Option<String>) -> Self {
        self.ipsec_tunnel = value;
        self
    }

    /// Set virtual_circuit
    pub fn set_virtual_circuit(mut self, value: Option<String>) -> Self {
        self.virtual_circuit = value;
        self
    }

    /// Set remote_peering_connection
    pub fn set_remote_peering_connection(mut self, value: Option<String>) -> Self {
        self.remote_peering_connection = value;
        self
    }

    /// Set vcn (unwraps Option)
    pub fn with_vcn(mut self, value: impl Into<String>) -> Self {
        self.vcn = Some(value.into());
        self
    }

    /// Set ipsec_tunnel (unwraps Option)
    pub fn with_ipsec_tunnel(mut self, value: impl Into<String>) -> Self {
        self.ipsec_tunnel = Some(value.into());
        self
    }

    /// Set virtual_circuit (unwraps Option)
    pub fn with_virtual_circuit(mut self, value: impl Into<String>) -> Self {
        self.virtual_circuit = Some(value.into());
        self
    }

    /// Set remote_peering_connection (unwraps Option)
    pub fn with_remote_peering_connection(mut self, value: impl Into<String>) -> Self {
        self.remote_peering_connection = Some(value.into());
        self
    }
}

impl Default for DefaultDrgRouteTables {
    fn default() -> Self {
        Self::new()
    }
}
