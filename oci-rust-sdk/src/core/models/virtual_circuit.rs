use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// For use with Oracle Cloud Infrastructure FastConnect. <p> A virtual circuit is an isolated network path that runs over one or more physical network connections to provide a single, logical connection between the edge router on the customer's existing network and Oracle Cloud Infrastructure. *Private* virtual circuits support private peering, and *public* virtual circuits support public peering. For more information, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm). <p> Each virtual circuit is made up of information shared between a customer, Oracle, and a provider (if the customer is using FastConnect via a provider). Who fills in a given property of a virtual circuit depends on whether the BGP session related to that virtual circuit goes from the customer's edge router to Oracle, or from the provider's edge router to Oracle. Also, in the case where the customer is using a provider, values for some of the properties may not be present immediately, but may get filled in as the provider and Oracle each do their part to provision the virtual circuit. <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuit {
    /// The provisioned data rate of the connection. To get a list of the available bandwidth levels (that is, shapes), see {@link #listFastConnectProviderVirtualCircuitBandwidthShapes(ListFastConnectProviderVirtualCircuitBandwidthShapesRequest) listFastConnectProviderVirtualCircuitBandwidthShapes}. <p> Example: {@code 10 Gbps}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_shape_name: Option<String>,

    /// Deprecated. Instead use the information in {@link FastConnectProviderService}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_management: Option<VirtualCircuitBgpManagement>,

    /// The state of the Ipv4 BGP session associated with the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_session_state: Option<VirtualCircuitBgpSessionState>,

    /// The state of the Ipv6 BGP session associated with the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_ipv6_session_state: Option<VirtualCircuitBgpIpv6SessionState>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// An array of mappings, each containing properties for a cross-connect or cross-connect group that is associated with this virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_mappings: Option<Vec<CrossConnectMapping>>,

    /// The routing policy sets how routing information about the Oracle cloud is shared over a public virtual circuit. Policies available are: {@code ORACLE_SERVICE_NETWORK}, {@code REGIONAL}, {@code MARKET_LEVEL}, and {@code GLOBAL}. See [Route Filtering](https://docs.oracle.com/iaas/Content/Network/Concepts/routingonprem.htm#route_filtering) for details. By default, routing information is shared for all routes in the same market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_policy: Option<Vec<VirtualCircuitRoutingPolicy>>,

    /// Set to {@code ENABLED} (the default) to activate the BGP session of the virtual circuit, set to {@code DISABLED} to deactivate the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_admin_state: Option<VirtualCircuitBgpAdminState>,

    /// Set to {@code true} to enable BFD for IPv4 BGP peering, or set to {@code false} to disable BFD. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bfd_enabled: Option<bool>,

    /// Set to {@code true} for the virtual circuit to carry only encrypted traffic, or set to {@code false} for the virtual circuit to carry unencrypted traffic. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_transport_mode: Option<bool>,

    /// Deprecated. Instead use {@code customerAsn}. If you specify values for both, the request will be rejected. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_asn: Option<i64>,

    /// The BGP ASN of the network at the other end of the BGP session from Oracle. If the session is between the customer's edge router and Oracle, the value is the customer's ASN. If the BGP session is between the provider's edge router and Oracle, the value is the provider's ASN. Can be a 2-byte or 4-byte ASN. Uses \"asplain\" format. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_asn: Option<i64>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the customer's {@link Drg} that this virtual circuit uses. Applicable only to private virtual circuits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,

    /// The virtual circuit's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The virtual circuit's current state. For information about the different states, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<VirtualCircuitLifecycleState>,

    /// The Oracle BGP ASN. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_bgp_asn: Option<i64>,

    /// Deprecated. Instead use {@code providerServiceId}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service offered by the provider (if the customer is connecting via a provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_id: Option<String>,

    /// The service key name offered by the provider (if the customer is connecting via a provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_key_name: Option<String>,

    /// Deprecated. Instead use {@code providerServiceId}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_name: Option<String>,

    /// The provider's state in relation to this virtual circuit (if the customer is connecting via a provider). ACTIVE means the provider has provisioned the virtual circuit from their end. INACTIVE means the provider has not yet provisioned the virtual circuit, or has de-provisioned it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_state: Option<VirtualCircuitProviderState>,

    /// For a public virtual circuit. The public IP prefixes (CIDRs) the customer wants to advertise across the connection. All prefix sizes are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_prefixes: Option<Vec<String>>,

    /// Provider-supplied reference information about this virtual circuit (if the customer is connecting via a provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_comment: Option<String>,

    /// The Oracle Cloud Infrastructure region where this virtual circuit is located.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Provider service type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<VirtualCircuitServiceType>,

    /// The date and time the virtual circuit was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// Whether the virtual circuit supports private or public peering. For more information, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm).
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<VirtualCircuitType>,

    /// The layer 3 IP MTU to use on this virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_mtu: Option<VirtualCircuitIpMtu>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_circuit_redundancy_metadata: Option<VirtualCircuitRedundancyMetadata>,
}

impl VirtualCircuit {
    /// Create a new VirtualCircuit
    pub fn new() -> Self {
        Self {
            bandwidth_shape_name: None,

            bgp_management: None,

            bgp_session_state: None,

            bgp_ipv6_session_state: None,

            compartment_id: None,

            cross_connect_mappings: None,

            routing_policy: None,

            bgp_admin_state: None,

            is_bfd_enabled: None,

            is_transport_mode: None,

            customer_bgp_asn: None,

            customer_asn: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            gateway_id: None,

            id: None,

            lifecycle_state: None,

            oracle_bgp_asn: None,

            provider_name: None,

            provider_service_id: None,

            provider_service_key_name: None,

            provider_service_name: None,

            provider_state: None,

            public_prefixes: None,

            reference_comment: None,

            region: None,

            service_type: None,

            time_created: None,

            r#type: None,

            ip_mtu: None,

            virtual_circuit_redundancy_metadata: None,
        }
    }

    /// Set bandwidth_shape_name
    pub fn set_bandwidth_shape_name(mut self, value: Option<String>) -> Self {
        self.bandwidth_shape_name = value;
        self
    }

    /// Set bgp_management
    pub fn set_bgp_management(mut self, value: Option<VirtualCircuitBgpManagement>) -> Self {
        self.bgp_management = value;
        self
    }

    /// Set bgp_session_state
    pub fn set_bgp_session_state(mut self, value: Option<VirtualCircuitBgpSessionState>) -> Self {
        self.bgp_session_state = value;
        self
    }

    /// Set bgp_ipv6_session_state
    pub fn set_bgp_ipv6_session_state(
        mut self,
        value: Option<VirtualCircuitBgpIpv6SessionState>,
    ) -> Self {
        self.bgp_ipv6_session_state = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set cross_connect_mappings
    pub fn set_cross_connect_mappings(mut self, value: Option<Vec<CrossConnectMapping>>) -> Self {
        self.cross_connect_mappings = value;
        self
    }

    /// Set routing_policy
    pub fn set_routing_policy(mut self, value: Option<Vec<VirtualCircuitRoutingPolicy>>) -> Self {
        self.routing_policy = value;
        self
    }

    /// Set bgp_admin_state
    pub fn set_bgp_admin_state(mut self, value: Option<VirtualCircuitBgpAdminState>) -> Self {
        self.bgp_admin_state = value;
        self
    }

    /// Set is_bfd_enabled
    pub fn set_is_bfd_enabled(mut self, value: Option<bool>) -> Self {
        self.is_bfd_enabled = value;
        self
    }

    /// Set is_transport_mode
    pub fn set_is_transport_mode(mut self, value: Option<bool>) -> Self {
        self.is_transport_mode = value;
        self
    }

    /// Set customer_bgp_asn
    pub fn set_customer_bgp_asn(mut self, value: Option<i64>) -> Self {
        self.customer_bgp_asn = value;
        self
    }

    /// Set customer_asn
    pub fn set_customer_asn(mut self, value: Option<i64>) -> Self {
        self.customer_asn = value;
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

    /// Set gateway_id
    pub fn set_gateway_id(mut self, value: Option<String>) -> Self {
        self.gateway_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<VirtualCircuitLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set oracle_bgp_asn
    pub fn set_oracle_bgp_asn(mut self, value: Option<i64>) -> Self {
        self.oracle_bgp_asn = value;
        self
    }

    /// Set provider_name
    pub fn set_provider_name(mut self, value: Option<String>) -> Self {
        self.provider_name = value;
        self
    }

    /// Set provider_service_id
    pub fn set_provider_service_id(mut self, value: Option<String>) -> Self {
        self.provider_service_id = value;
        self
    }

    /// Set provider_service_key_name
    pub fn set_provider_service_key_name(mut self, value: Option<String>) -> Self {
        self.provider_service_key_name = value;
        self
    }

    /// Set provider_service_name
    pub fn set_provider_service_name(mut self, value: Option<String>) -> Self {
        self.provider_service_name = value;
        self
    }

    /// Set provider_state
    pub fn set_provider_state(mut self, value: Option<VirtualCircuitProviderState>) -> Self {
        self.provider_state = value;
        self
    }

    /// Set public_prefixes
    pub fn set_public_prefixes(mut self, value: Option<Vec<String>>) -> Self {
        self.public_prefixes = value;
        self
    }

    /// Set reference_comment
    pub fn set_reference_comment(mut self, value: Option<String>) -> Self {
        self.reference_comment = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: Option<String>) -> Self {
        self.region = value;
        self
    }

    /// Set service_type
    pub fn set_service_type(mut self, value: Option<VirtualCircuitServiceType>) -> Self {
        self.service_type = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<VirtualCircuitType>) -> Self {
        self.r#type = value;
        self
    }

    /// Set ip_mtu
    pub fn set_ip_mtu(mut self, value: Option<VirtualCircuitIpMtu>) -> Self {
        self.ip_mtu = value;
        self
    }

    /// Set virtual_circuit_redundancy_metadata
    pub fn set_virtual_circuit_redundancy_metadata(
        mut self,
        value: Option<VirtualCircuitRedundancyMetadata>,
    ) -> Self {
        self.virtual_circuit_redundancy_metadata = value;
        self
    }

    /// Set bandwidth_shape_name (unwraps Option)
    pub fn with_bandwidth_shape_name(mut self, value: impl Into<String>) -> Self {
        self.bandwidth_shape_name = Some(value.into());
        self
    }

    /// Set bgp_management (unwraps Option)
    pub fn with_bgp_management(mut self, value: VirtualCircuitBgpManagement) -> Self {
        self.bgp_management = Some(value);
        self
    }

    /// Set bgp_session_state (unwraps Option)
    pub fn with_bgp_session_state(mut self, value: VirtualCircuitBgpSessionState) -> Self {
        self.bgp_session_state = Some(value);
        self
    }

    /// Set bgp_ipv6_session_state (unwraps Option)
    pub fn with_bgp_ipv6_session_state(mut self, value: VirtualCircuitBgpIpv6SessionState) -> Self {
        self.bgp_ipv6_session_state = Some(value);
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set cross_connect_mappings (unwraps Option)
    pub fn with_cross_connect_mappings(mut self, value: Vec<CrossConnectMapping>) -> Self {
        self.cross_connect_mappings = Some(value);
        self
    }

    /// Set routing_policy (unwraps Option)
    pub fn with_routing_policy(mut self, value: Vec<VirtualCircuitRoutingPolicy>) -> Self {
        self.routing_policy = Some(value);
        self
    }

    /// Set bgp_admin_state (unwraps Option)
    pub fn with_bgp_admin_state(mut self, value: VirtualCircuitBgpAdminState) -> Self {
        self.bgp_admin_state = Some(value);
        self
    }

    /// Set is_bfd_enabled (unwraps Option)
    pub fn with_is_bfd_enabled(mut self, value: bool) -> Self {
        self.is_bfd_enabled = Some(value);
        self
    }

    /// Set is_transport_mode (unwraps Option)
    pub fn with_is_transport_mode(mut self, value: bool) -> Self {
        self.is_transport_mode = Some(value);
        self
    }

    /// Set customer_bgp_asn (unwraps Option)
    pub fn with_customer_bgp_asn(mut self, value: i64) -> Self {
        self.customer_bgp_asn = Some(value);
        self
    }

    /// Set customer_asn (unwraps Option)
    pub fn with_customer_asn(mut self, value: i64) -> Self {
        self.customer_asn = Some(value);
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

    /// Set gateway_id (unwraps Option)
    pub fn with_gateway_id(mut self, value: impl Into<String>) -> Self {
        self.gateway_id = Some(value.into());
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: VirtualCircuitLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set oracle_bgp_asn (unwraps Option)
    pub fn with_oracle_bgp_asn(mut self, value: i64) -> Self {
        self.oracle_bgp_asn = Some(value);
        self
    }

    /// Set provider_name (unwraps Option)
    pub fn with_provider_name(mut self, value: impl Into<String>) -> Self {
        self.provider_name = Some(value.into());
        self
    }

    /// Set provider_service_id (unwraps Option)
    pub fn with_provider_service_id(mut self, value: impl Into<String>) -> Self {
        self.provider_service_id = Some(value.into());
        self
    }

    /// Set provider_service_key_name (unwraps Option)
    pub fn with_provider_service_key_name(mut self, value: impl Into<String>) -> Self {
        self.provider_service_key_name = Some(value.into());
        self
    }

    /// Set provider_service_name (unwraps Option)
    pub fn with_provider_service_name(mut self, value: impl Into<String>) -> Self {
        self.provider_service_name = Some(value.into());
        self
    }

    /// Set provider_state (unwraps Option)
    pub fn with_provider_state(mut self, value: VirtualCircuitProviderState) -> Self {
        self.provider_state = Some(value);
        self
    }

    /// Set public_prefixes (unwraps Option)
    pub fn with_public_prefixes(mut self, value: Vec<String>) -> Self {
        self.public_prefixes = Some(value);
        self
    }

    /// Set reference_comment (unwraps Option)
    pub fn with_reference_comment(mut self, value: impl Into<String>) -> Self {
        self.reference_comment = Some(value.into());
        self
    }

    /// Set region (unwraps Option)
    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    /// Set service_type (unwraps Option)
    pub fn with_service_type(mut self, value: VirtualCircuitServiceType) -> Self {
        self.service_type = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: VirtualCircuitType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Set ip_mtu (unwraps Option)
    pub fn with_ip_mtu(mut self, value: VirtualCircuitIpMtu) -> Self {
        self.ip_mtu = Some(value);
        self
    }

    /// Set virtual_circuit_redundancy_metadata (unwraps Option)
    pub fn with_virtual_circuit_redundancy_metadata(
        mut self,
        value: VirtualCircuitRedundancyMetadata,
    ) -> Self {
        self.virtual_circuit_redundancy_metadata = Some(value);
        self
    }
}

impl Default for VirtualCircuit {
    fn default() -> Self {
        Self::new()
    }
}
