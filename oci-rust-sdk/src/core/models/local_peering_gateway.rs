use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A local peering gateway (LPG) is an object on a VCN that lets that VCN peer with another VCN in the same region. *Peering* means that the two VCNs can communicate using private IP addresses, but without the traffic traversing the internet or routing through your on-premises network. For more information, see [VCN Peering](https://docs.oracle.com/iaas/Content/Network/Tasks/VCNpeering.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalPeeringGateway {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the LPG.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The LPG's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// Whether the VCN at the other end of the peering is in a different tenancy. <p> Example: {@code false}
    pub is_cross_tenancy_peering: bool,

    /// The LPG's current lifecycle state.
    pub lifecycle_state: LocalPeeringGatewayLifecycleState,

    /// Whether the LPG is peered with another LPG. {@code NEW} means the LPG has not yet been peered. {@code PENDING} means the peering is being established. {@code REVOKED} means the LPG at the other end of the peering has been deleted.
    pub peering_status: LocalPeeringGatewayPeeringStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the peered LPG.
    pub peer_id: String,

    /// The date and time the LPG was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN that uses the LPG.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The smallest aggregate CIDR that contains all the CIDR routes advertised by the VCN at the other end of the peering from this LPG. See {@code peerAdvertisedCidrDetails} for the individual CIDRs. The value is {@code null} if the LPG is not peered. <p> Example: {@code 192.168.0.0/16}, or if aggregated with {@code 172.16.0.0/24} then {@code 128.0.0.0/1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_advertised_cidr: Option<String>,

    /// The specific ranges of IP addresses available on or via the VCN at the other end of the peering from this LPG. The value is {@code null} if the LPG is not peered. You can use these as destination CIDRs for route rules to route a subnet's traffic to this LPG. <p> Example: [{@code 192.168.0.0/16}, {@code 172.16.0.0/24}]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_advertised_cidr_details: Option<Vec<String>>,

    /// Additional information regarding the peering status, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peering_status_details: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the LPG is using. <p> For information about why you would associate a route table with an LPG, see [Transit Routing: Access to Multiple VCNs in Same Region](https://docs.oracle.com/iaas/Content/Network/Tasks/transitrouting.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for LocalPeeringGateway
pub struct LocalPeeringGatewayRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the LPG.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The LPG's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// Whether the VCN at the other end of the peering is in a different tenancy. <p> Example: {@code false}
    pub is_cross_tenancy_peering: bool,

    /// The LPG's current lifecycle state.
    pub lifecycle_state: LocalPeeringGatewayLifecycleState,

    /// Whether the LPG is peered with another LPG. {@code NEW} means the LPG has not yet been peered. {@code PENDING} means the peering is being established. {@code REVOKED} means the LPG at the other end of the peering has been deleted.
    pub peering_status: LocalPeeringGatewayPeeringStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the peered LPG.
    pub peer_id: String,

    /// The date and time the LPG was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN that uses the LPG.
    pub vcn_id: String,
}

impl LocalPeeringGateway {
    /// Create a new LocalPeeringGateway with required fields
    pub fn new(required: LocalPeeringGatewayRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            is_cross_tenancy_peering: required.is_cross_tenancy_peering,

            lifecycle_state: required.lifecycle_state,

            peering_status: required.peering_status,

            peer_id: required.peer_id,

            time_created: required.time_created,

            vcn_id: required.vcn_id,

            defined_tags: None,

            freeform_tags: None,

            security_attributes: None,

            peer_advertised_cidr: None,

            peer_advertised_cidr_details: None,

            peering_status_details: None,

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
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set security_attributes
    pub fn set_security_attributes(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.security_attributes = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set is_cross_tenancy_peering
    pub fn set_is_cross_tenancy_peering(mut self, value: bool) -> Self {
        self.is_cross_tenancy_peering = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: LocalPeeringGatewayLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set peer_advertised_cidr
    pub fn set_peer_advertised_cidr(mut self, value: Option<String>) -> Self {
        self.peer_advertised_cidr = value;
        self
    }

    /// Set peer_advertised_cidr_details
    pub fn set_peer_advertised_cidr_details(mut self, value: Option<Vec<String>>) -> Self {
        self.peer_advertised_cidr_details = value;
        self
    }

    /// Set peering_status
    pub fn set_peering_status(mut self, value: LocalPeeringGatewayPeeringStatus) -> Self {
        self.peering_status = value;
        self
    }

    /// Set peering_status_details
    pub fn set_peering_status_details(mut self, value: Option<String>) -> Self {
        self.peering_status_details = value;
        self
    }

    /// Set peer_id
    pub fn set_peer_id(mut self, value: String) -> Self {
        self.peer_id = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
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

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set security_attributes (unwraps Option)
    pub fn with_security_attributes(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.security_attributes = Some(value);
        self
    }

    /// Set peer_advertised_cidr (unwraps Option)
    pub fn with_peer_advertised_cidr(mut self, value: impl Into<String>) -> Self {
        self.peer_advertised_cidr = Some(value.into());
        self
    }

    /// Set peer_advertised_cidr_details (unwraps Option)
    pub fn with_peer_advertised_cidr_details(mut self, value: Vec<String>) -> Self {
        self.peer_advertised_cidr_details = Some(value);
        self
    }

    /// Set peering_status_details (unwraps Option)
    pub fn with_peering_status_details(mut self, value: impl Into<String>) -> Self {
        self.peering_status_details = Some(value.into());
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }
}
