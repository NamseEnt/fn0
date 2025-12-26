use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualCircuitDetails {
    /// The provisioned data rate of the connection. To get a list of the available bandwidth levels (that is, shapes), see {@link #listFastConnectProviderVirtualCircuitBandwidthShapes(ListFastConnectProviderVirtualCircuitBandwidthShapesRequest) listFastConnectProviderVirtualCircuitBandwidthShapes}. To be updated only by the customer who owns the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_shape_name: Option<String>,

    /// An array of mappings, each containing properties for a cross-connect or cross-connect group associated with this virtual circuit. <p> The customer and provider can update different properties in the mapping depending on the situation. See the description of the {@link CrossConnectMapping}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_mappings: Option<Vec<CrossConnectMapping>>,

    /// The routing policy sets how routing information about the Oracle cloud is shared over a public virtual circuit. Policies available are: {@code ORACLE_SERVICE_NETWORK}, {@code REGIONAL}, {@code MARKET_LEVEL}, and {@code GLOBAL}. See [Route Filtering](https://docs.oracle.com/iaas/Content/Network/Concepts/routingonprem.htm#route_filtering) for details. By default, routing information is shared for all routes in the same market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_policy: Option<Vec<UpdateVirtualCircuitDetailsRoutingPolicy>>,

    /// Set to {@code ENABLED} (the default) to activate the BGP session of the virtual circuit, set to {@code DISABLED} to deactivate the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_admin_state: Option<UpdateVirtualCircuitDetailsBgpAdminState>,

    /// Set to {@code true} to enable BFD for IPv4 BGP peering, or set to {@code false} to disable BFD. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bfd_enabled: Option<bool>,

    /// Set to {@code true} for the virtual circuit to carry only encrypted traffic, or set to {@code false} for the virtual circuit to carry unencrypted traffic. If this is not set, the default is {@code false}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_transport_mode: Option<bool>,

    /// Deprecated. Instead use {@code customerAsn}. If you specify values for both, the request will be rejected. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_asn: Option<i64>,

    /// The BGP ASN of the network at the other end of the BGP session from Oracle. <p> If the BGP session is from the customer's edge router to Oracle, the required value is the customer's ASN, and it can be updated only by the customer. <p> If the BGP session is from the provider's edge router to Oracle, the required value is the provider's ASN, and it can be updated only by the provider. <p> Can be a 2-byte or 4-byte ASN. Uses \"asplain\" format. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
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

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Drg} that this private virtual circuit uses. <p> To be updated only by the customer who owns the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,

    /// The provider's state in relation to this virtual circuit. Relevant only if the customer is using FastConnect via a provider. ACTIVE means the provider has provisioned the virtual circuit from their end. INACTIVE means the provider has not yet provisioned the virtual circuit, or has de-provisioned it. <p> To be updated only by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_state: Option<UpdateVirtualCircuitDetailsProviderState>,

    /// The service key name offered by the provider (if the customer is connecting via a provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_service_key_name: Option<String>,

    /// Provider-supplied reference information about this virtual circuit. Relevant only if the customer is using FastConnect via a provider. <p> To be updated only by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_comment: Option<String>,

    /// The layer 3 IP MTU to use on this virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_mtu: Option<VirtualCircuitIpMtu>,
}

impl UpdateVirtualCircuitDetails {
    /// Create a new UpdateVirtualCircuitDetails
    pub fn new() -> Self {
        Self {
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

            provider_state: None,

            provider_service_key_name: None,

            reference_comment: None,

            ip_mtu: None,
        }
    }

    /// Set bandwidth_shape_name
    pub fn set_bandwidth_shape_name(mut self, value: Option<String>) -> Self {
        self.bandwidth_shape_name = value;
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
        value: Option<Vec<UpdateVirtualCircuitDetailsRoutingPolicy>>,
    ) -> Self {
        self.routing_policy = value;
        self
    }

    /// Set bgp_admin_state
    pub fn set_bgp_admin_state(
        mut self,
        value: Option<UpdateVirtualCircuitDetailsBgpAdminState>,
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

    /// Set provider_state
    pub fn set_provider_state(
        mut self,
        value: Option<UpdateVirtualCircuitDetailsProviderState>,
    ) -> Self {
        self.provider_state = value;
        self
    }

    /// Set provider_service_key_name
    pub fn set_provider_service_key_name(mut self, value: Option<String>) -> Self {
        self.provider_service_key_name = value;
        self
    }

    /// Set reference_comment
    pub fn set_reference_comment(mut self, value: Option<String>) -> Self {
        self.reference_comment = value;
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
        value: Vec<UpdateVirtualCircuitDetailsRoutingPolicy>,
    ) -> Self {
        self.routing_policy = Some(value);
        self
    }

    /// Set bgp_admin_state (unwraps Option)
    pub fn with_bgp_admin_state(mut self, value: UpdateVirtualCircuitDetailsBgpAdminState) -> Self {
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

    /// Set provider_state (unwraps Option)
    pub fn with_provider_state(mut self, value: UpdateVirtualCircuitDetailsProviderState) -> Self {
        self.provider_state = Some(value);
        self
    }

    /// Set provider_service_key_name (unwraps Option)
    pub fn with_provider_service_key_name(mut self, value: impl Into<String>) -> Self {
        self.provider_service_key_name = Some(value.into());
        self
    }

    /// Set reference_comment (unwraps Option)
    pub fn with_reference_comment(mut self, value: impl Into<String>) -> Self {
        self.reference_comment = Some(value.into());
        self
    }

    /// Set ip_mtu (unwraps Option)
    pub fn with_ip_mtu(mut self, value: VirtualCircuitIpMtu) -> Self {
        self.ip_mtu = Some(value);
        self
    }
}

impl Default for UpdateVirtualCircuitDetails {
    fn default() -> Self {
        Self::new()
    }
}
