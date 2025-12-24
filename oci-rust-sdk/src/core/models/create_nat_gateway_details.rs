use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNatGatewayDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the NAT gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the gateway belongs to.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Whether the NAT gateway blocks traffic through it. The default is {@code false}. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_traffic: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP address associated with the NAT gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table used by the NAT gateway. <p> If you don't specify a route table here, the NAT gateway is created without an associated route table. The Networking service does NOT automatically associate the attached VCN's default route table with the NAT gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for CreateNatGatewayDetails
pub struct CreateNatGatewayDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the NAT gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the gateway belongs to.
    pub vcn_id: String,
}

impl CreateNatGatewayDetails {
    /// Create a new CreateNatGatewayDetails with required fields
    pub fn new(required: CreateNatGatewayDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            block_traffic: None,

            public_ip_id: None,

            route_table_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set block_traffic
    pub fn set_block_traffic(mut self, value: Option<bool>) -> Self {
        self.block_traffic = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set public_ip_id
    pub fn set_public_ip_id(mut self, value: Option<String>) -> Self {
        self.public_ip_id = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set block_traffic (unwraps Option)
    pub fn with_block_traffic(mut self, value: bool) -> Self {
        self.block_traffic = Some(value);
        self
    }

    /// Set public_ip_id (unwraps Option)
    pub fn with_public_ip_id(mut self, value: impl Into<String>) -> Self {
        self.public_ip_id = Some(value.into());
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }
}
