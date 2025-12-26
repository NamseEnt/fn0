use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecConnectionTunnelSharedSecretRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub tunnel_id: String,

    /// Details object for updating a IPSec connection tunnel's sharedSecret.
    pub update_ipsec_connection_tunnel_shared_secret_details: UpdateIPSecConnectionTunnelSharedSecretDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateIPSecConnectionTunnelSharedSecretRequest
pub struct UpdateIPSecConnectionTunnelSharedSecretRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub tunnel_id: String,

    /// Details object for updating a IPSec connection tunnel's sharedSecret.
    pub update_ipsec_connection_tunnel_shared_secret_details: UpdateIPSecConnectionTunnelSharedSecretDetails,
}

impl UpdateIPSecConnectionTunnelSharedSecretRequest {
    /// Create a new UpdateIPSecConnectionTunnelSharedSecretRequest with required fields
    pub fn new(required: UpdateIPSecConnectionTunnelSharedSecretRequestRequired) -> Self {
        Self {
            ipsc_id: required.ipsc_id,

            tunnel_id: required.tunnel_id,

            update_ipsec_connection_tunnel_shared_secret_details: required.update_ipsec_connection_tunnel_shared_secret_details,

            if_match: None,
}
    }

    /// Set ipsc_id
    pub fn set_ipsc_id(mut self, value: String) -> Self {
        self.ipsc_id = value;
        self
    }

    /// Set tunnel_id
    pub fn set_tunnel_id(mut self, value: String) -> Self {
        self.tunnel_id = value;
        self
    }

    /// Set update_ipsec_connection_tunnel_shared_secret_details
    pub fn set_update_ipsec_connection_tunnel_shared_secret_details(mut self, value: UpdateIPSecConnectionTunnelSharedSecretDetails) -> Self {
        self.update_ipsec_connection_tunnel_shared_secret_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


