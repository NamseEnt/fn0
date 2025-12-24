use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the update details for the VCN attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VcnDrgAttachmentNetworkUpdateDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// This is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table that is used to route the traffic as it enters a VCN through this attachment. <p> For information about why you would associate a route table with a DRG attachment, see: <p> [Transit Routing: Access to Multiple VCNs in Same Region](https://docs.oracle.com/iaas/Content/Network/Tasks/transitrouting.htm) * [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// Indicates whether the VCN CIDRs or the individual subnet CIDRs are imported from the attachment. Routes from the VCN ingress route table are always imported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_route_type: Option<String>,
}

/// Required fields for VcnDrgAttachmentNetworkUpdateDetails
pub struct VcnDrgAttachmentNetworkUpdateDetailsRequired {
    pub r#type: String,
}

impl VcnDrgAttachmentNetworkUpdateDetails {
    /// Create a new VcnDrgAttachmentNetworkUpdateDetails with required fields
    pub fn new(required: VcnDrgAttachmentNetworkUpdateDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            route_table_id: None,

            vcn_route_type: None,
        }
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set vcn_route_type
    pub fn set_vcn_route_type(mut self, value: Option<String>) -> Self {
        self.vcn_route_type = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set vcn_route_type (unwraps Option)
    pub fn with_vcn_route_type(mut self, value: impl Into<String>) -> Self {
        self.vcn_route_type = Some(value.into());
        self
    }
}
