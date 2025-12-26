use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVirtualCircuitDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the virtual circuit.
    pub compartment_id: String,

    /// The type of IP addresses used in this virtual circuit. PRIVATE means [RFC 1918](https://tools.ietf.org/html/rfc1918) addresses (10.0.0.0/8, 172.16/12, and 192.168/16).
    #[serde(rename = "type")]
    pub r#type: CreateVirtualCircuitDetailsType,

    /// The provisioned data rate of the connection. To get a list of the available bandwidth levels (that is, shapes), see {@link #listFastConnectProviderVirtualCircuitBandwidthShapes(ListFastConnectProviderVirtualCircuitBandwidthShapesRequest) listFastConnectProviderVirtualCircuitBandwidthShapes}. <p> Example: {@code 10 Gbps}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_shape_name: Option<String>,

    /// Create a {@code CrossConnectMapping} for each cross-connect or cross-connect group this virtual circuit will run on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_mappings: Option<Vec<CrossConnectMapping>>,

    /// The routing policy sets how routing information about the Oracle cloud is shared over a public virtual circuit. Policies available are: {@code ORACLE_SERVICE_NETWORK}, {@code REGIONAL}, {@code MARKET_LEVEL}, and {@code GLOBAL}. See [Route Filtering](https://docs.oracle.com/iaas/Content/Network/Concepts/routingonprem.htm#route_filtering) for details. By default, routing information is shared for all routes in the same market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_policy: Option<Vec<CreateVirtualCircuitDetailsRoutingPolicy>>,

    /// Set to {@code ENABLED} (the default) to activate the BGP session of the virtual circuit, set to {@code DISABLED} to deactivate the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_admin_state: Option<CreateVirtualCircuitDetailsBgpAdminState>,

    /// Set to {@code true} to enable BFD for IPv4 BGP peering, or set to {@code false} to disable BFD. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bfd_enabled: Option<bool>,

    /// Set to {@code true} for the virtual circuit to carry only encrypted traffic, or set to {@code false} for the virtual circuit to carry unencrypted traffic. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_transport_mode: Option<bool>,

    /// Deprecated. Instead use {@code customerAsn}. If you specify values for both, the request will be rejected. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_asn: Option<i64>,

    /// Your BGP ASN (either public or private). Provide this value only if there's a BGP session that goes from your edge router to Oracle. Otherwise, leave this empty or null. Can be a 2-byte or 4-byte ASN. Uses \"asplain\" format. <p> Example: {@code 12345} (2-byte) or {@code 1587232876} (4-byte) Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
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

    /// For private virtual circuits only. The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Drg} that this virtual circuit uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,

    /// Deprecated. Instead use {@code providerServiceId}. To get a list of the provider names, see {@link #listFastConnectProviderServices(ListFastConnectProviderServicesRequest) listFastConnectProviderServices}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service offered by the provider (if you're connecting via a provider). To get a list of the available service offerings, see {@link #listFastConnectProviderServices(ListFastConnectProviderServicesRequest) listFastConnectProviderServices}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_id: Option<String>,

    /// The service key name offered by the provider (if the customer is connecting via a provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_key_name: Option<String>,

    /// Deprecated. Instead use {@code providerServiceId}. To get a list of the provider names, see {@link #listFastConnectProviderServices(ListFastConnectProviderServicesRequest) listFastConnectProviderServices}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_name: Option<String>,

    /// For a public virtual circuit. The public IP prefixes (CIDRs) the customer wants to advertise across the connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_prefixes: Option<Vec<CreateVirtualCircuitPublicPrefixDetails>>,

    /// The Oracle Cloud Infrastructure region where this virtual circuit is located. Example: {@code phx}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The layer 3 IP MTU to use with this virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_mtu: Option<VirtualCircuitIpMtu>,
}

/// Required fields for CreateVirtualCircuitDetails
pub struct CreateVirtualCircuitDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the virtual circuit.
    pub compartment_id: String,

    /// The type of IP addresses used in this virtual circuit. PRIVATE means [RFC 1918](https://tools.ietf.org/html/rfc1918) addresses (10.0.0.0/8, 172.16/12, and 192.168/16).
    pub r#type: CreateVirtualCircuitDetailsType,
}

impl CreateVirtualCircuitDetails {
    /// Create a new CreateVirtualCircuitDetails with required fields
    pub fn new(required: CreateVirtualCircuitDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            r#type: required.r#type,

            bandwidth_shape_name: None,

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

            provider_name: None,

            provider_service_id: None,

            provider_service_key_name: None,

            provider_service_name: None,

            public_prefixes: None,

            region: None,

            ip_mtu: None,
        }
    }

    /// Set bandwidth_shape_name
    pub fn set_bandwidth_shape_name(mut self, value: Option<String>) -> Self {
        self.bandwidth_shape_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set cross_connect_mappings
    pub fn set_cross_connect_mappings(mut self, value: Option<Vec<CrossConnectMapping>>) -> Self {
        self.cross_connect_mappings = value;
        self
    }

    /// Set routing_policy
    pub fn set_routing_policy(
        mut self,
        value: Option<Vec<CreateVirtualCircuitDetailsRoutingPolicy>>,
    ) -> Self {
        self.routing_policy = value;
        self
    }

    /// Set bgp_admin_state
    pub fn set_bgp_admin_state(
        mut self,
        value: Option<CreateVirtualCircuitDetailsBgpAdminState>,
    ) -> Self {
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

    /// Set public_prefixes
    pub fn set_public_prefixes(
        mut self,
        value: Option<Vec<CreateVirtualCircuitPublicPrefixDetails>>,
    ) -> Self {
        self.public_prefixes = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: Option<String>) -> Self {
        self.region = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: CreateVirtualCircuitDetailsType) -> Self {
        self.r#type = value;
        self
    }

    /// Set ip_mtu
    pub fn set_ip_mtu(mut self, value: Option<VirtualCircuitIpMtu>) -> Self {
        self.ip_mtu = value;
        self
    }

    /// Set bandwidth_shape_name (unwraps Option)
    pub fn with_bandwidth_shape_name(mut self, value: impl Into<String>) -> Self {
        self.bandwidth_shape_name = Some(value.into());
        self
    }

    /// Set cross_connect_mappings (unwraps Option)
    pub fn with_cross_connect_mappings(mut self, value: Vec<CrossConnectMapping>) -> Self {
        self.cross_connect_mappings = Some(value);
        self
    }

    /// Set routing_policy (unwraps Option)
    pub fn with_routing_policy(
        mut self,
        value: Vec<CreateVirtualCircuitDetailsRoutingPolicy>,
    ) -> Self {
        self.routing_policy = Some(value);
        self
    }

    /// Set bgp_admin_state (unwraps Option)
    pub fn with_bgp_admin_state(mut self, value: CreateVirtualCircuitDetailsBgpAdminState) -> Self {
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

    /// Set public_prefixes (unwraps Option)
    pub fn with_public_prefixes(
        mut self,
        value: Vec<CreateVirtualCircuitPublicPrefixDetails>,
    ) -> Self {
        self.public_prefixes = Some(value);
        self
    }

    /// Set region (unwraps Option)
    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    /// Set ip_mtu (unwraps Option)
    pub fn with_ip_mtu(mut self, value: VirtualCircuitIpMtu) -> Self {
        self.ip_mtu = Some(value);
        self
    }
}
