use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A NAT (Network Address Translation) gateway, which represents a router that lets instances without public IPs contact the public internet without exposing the instance to inbound internet traffic. For more information, see [NAT Gateway](https://docs.oracle.com/iaas/Content/Network/Tasks/NATgateway.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you are not authorized, talk to an administrator. If you are an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatGateway {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the NAT gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the NAT gateway.
    pub id: String,

    /// Whether the NAT gateway blocks traffic through it. The default is {@code false}. <p> Example: {@code true}
    pub block_traffic: bool,

    /// The NAT gateway's current state.
    pub lifecycle_state: NatGatewayLifecycleState,

    /// The IP address associated with the NAT gateway.
    pub nat_ip: String,

    /// The date and time the NAT gateway was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the NAT gateway belongs to.
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

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP address associated with the NAT gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table used by the NAT gateway. <p> If you don't specify a route table here, the NAT gateway is created without an associated route table. The Networking service does NOT automatically associate the attached VCN's default route table with the NAT gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for NatGateway
pub struct NatGatewayRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the NAT gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the NAT gateway.
    pub id: String,

    /// Whether the NAT gateway blocks traffic through it. The default is {@code false}. <p> Example: {@code true}
    pub block_traffic: bool,

    /// The NAT gateway's current state.
    pub lifecycle_state: NatGatewayLifecycleState,

    /// The IP address associated with the NAT gateway.
    pub nat_ip: String,

    /// The date and time the NAT gateway was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the NAT gateway belongs to.
    pub vcn_id: String,
}

impl NatGateway {
    /// Create a new NatGateway with required fields
    pub fn new(required: NatGatewayRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            block_traffic: required.block_traffic,

            lifecycle_state: required.lifecycle_state,

            nat_ip: required.nat_ip,

            time_created: required.time_created,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set block_traffic
    pub fn set_block_traffic(mut self, value: bool) -> Self {
        self.block_traffic = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: NatGatewayLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set nat_ip
    pub fn set_nat_ip(mut self, value: String) -> Self {
        self.nat_ip = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
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
