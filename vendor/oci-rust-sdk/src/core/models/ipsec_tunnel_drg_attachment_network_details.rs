use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the IPSec tunnel attached to the DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpsecTunnelDrgAttachmentNetworkDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The IPSec connection that contains the attached IPSec tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec_connection_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the virtual circuit's DRG attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_attachment_id: Option<String>,
}

/// Required fields for IpsecTunnelDrgAttachmentNetworkDetails
pub struct IpsecTunnelDrgAttachmentNetworkDetailsRequired {
    pub r#type: String,
}

impl IpsecTunnelDrgAttachmentNetworkDetails {
    /// Create a new IpsecTunnelDrgAttachmentNetworkDetails with required fields
    pub fn new(required: IpsecTunnelDrgAttachmentNetworkDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            ipsec_connection_id: None,

            transport_attachment_id: None,
        }
    }

    /// Set ipsec_connection_id
    pub fn set_ipsec_connection_id(mut self, value: Option<String>) -> Self {
        self.ipsec_connection_id = value;
        self
    }

    /// Set transport_attachment_id
    pub fn set_transport_attachment_id(mut self, value: Option<String>) -> Self {
        self.transport_attachment_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set ipsec_connection_id (unwraps Option)
    pub fn with_ipsec_connection_id(mut self, value: impl Into<String>) -> Self {
        self.ipsec_connection_id = Some(value.into());
        self
    }

    /// Set transport_attachment_id (unwraps Option)
    pub fn with_transport_attachment_id(mut self, value: impl Into<String>) -> Self {
        self.transport_attachment_id = Some(value.into());
        self
    }
}
