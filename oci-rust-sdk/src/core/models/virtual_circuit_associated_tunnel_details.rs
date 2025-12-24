use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Detailed private tunnel info associated with the virtual circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuitAssociatedTunnelDetails {
    /// The type of the tunnel associated with the virtual circuit.
    pub tunnel_type: VirtualCircuitAssociatedTunnelDetailsTunnelType,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec tunnel associated with the virtual circuit.
    pub tunnel_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of IPSec connection associated with the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec_connection_id: Option<String>,
}

/// Required fields for VirtualCircuitAssociatedTunnelDetails
pub struct VirtualCircuitAssociatedTunnelDetailsRequired {
    /// The type of the tunnel associated with the virtual circuit.
    pub tunnel_type: VirtualCircuitAssociatedTunnelDetailsTunnelType,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec tunnel associated with the virtual circuit.
    pub tunnel_id: String,
}

impl VirtualCircuitAssociatedTunnelDetails {
    /// Create a new VirtualCircuitAssociatedTunnelDetails with required fields
    pub fn new(required: VirtualCircuitAssociatedTunnelDetailsRequired) -> Self {
        Self {
            tunnel_type: required.tunnel_type,

            tunnel_id: required.tunnel_id,

            ipsec_connection_id: None,
        }
    }

    /// Set tunnel_type
    pub fn set_tunnel_type(
        mut self,
        value: VirtualCircuitAssociatedTunnelDetailsTunnelType,
    ) -> Self {
        self.tunnel_type = value;
        self
    }

    /// Set ipsec_connection_id
    pub fn set_ipsec_connection_id(mut self, value: Option<String>) -> Self {
        self.ipsec_connection_id = value;
        self
    }

    /// Set tunnel_id
    pub fn set_tunnel_id(mut self, value: String) -> Self {
        self.tunnel_id = value;
        self
    }

    /// Set ipsec_connection_id (unwraps Option)
    pub fn with_ipsec_connection_id(mut self, value: impl Into<String>) -> Self {
        self.ipsec_connection_id = Some(value.into());
        self
    }
}
