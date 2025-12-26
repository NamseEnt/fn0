use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A DRG route rule is a mapping between a destination IP address range and a DRG attachment. The map is used to route matching packets. Traffic will be routed across the attachments using Equal-cost multi-path routing (ECMP) if there are multiple rules with identical destinations and none of the rules conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRouteRule {
    /// Represents the range of IP addresses to match against when routing traffic. <p> Potential values: * An IP address range (IPv4 or IPv6) in CIDR notation. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}. * When you're setting up a security rule for traffic destined for a particular {@code Service} through a service gateway, this is the {@code cidrBlock} value associated with that {@link Service}. For example: {@code oci-phx-objectstorage}.
    pub destination: String,

    /// The type of destination for the rule. <p> Allowed values: <p> {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation. * {@code SERVICE_CIDR_BLOCK}: If the rule's {@code destination} is the {@code cidrBlock} value for a {@link Service} (the rule is for traffic destined for a particular {@code Service} through a service gateway).
    pub destination_type: DrgRouteRuleDestinationType,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the next hop DRG attachment responsible for reaching the network destination. <p> A value of {@code BLACKHOLE} means traffic for this route is discarded without notification.
    pub next_hop_drg_attachment_id: String,

    /// The Oracle-assigned ID of the DRG route rule.
    pub id: String,

    /// The earliest origin of a route. If a route is advertised to a DRG through an IPsec tunnel attachment, and is propagated to peered DRGs via RPC attachments, the route's provenance in the peered DRGs remains {@code IPSEC_TUNNEL}, because that is the earliest origin. <p> No routes with a provenance {@code IPSEC_TUNNEL} or {@code VIRTUAL_CIRCUIT} will be exported to IPsec tunnel or virtual circuit attachments, regardless of the attachment's export distribution.
    pub route_provenance: DrgRouteRuleRouteProvenance,

    /// You can specify static routes for the DRG route table using the API. The DRG learns dynamic routes from the DRG attachments using various routing protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_type: Option<DrgRouteRuleRouteType>,

    /// Indicates that the route was not imported due to a conflict between route rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_conflict: Option<bool>,

    /// Indicates that if the next hop attachment does not exist, so traffic for this route is discarded without notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blackhole: Option<bool>,

    /// Additional properties for the route, computed by the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
}

/// Required fields for DrgRouteRule
pub struct DrgRouteRuleRequired {
    /// Represents the range of IP addresses to match against when routing traffic. <p> Potential values: * An IP address range (IPv4 or IPv6) in CIDR notation. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}. * When you're setting up a security rule for traffic destined for a particular {@code Service} through a service gateway, this is the {@code cidrBlock} value associated with that {@link Service}. For example: {@code oci-phx-objectstorage}.
    pub destination: String,

    /// The type of destination for the rule. <p> Allowed values: <p> {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation. * {@code SERVICE_CIDR_BLOCK}: If the rule's {@code destination} is the {@code cidrBlock} value for a {@link Service} (the rule is for traffic destined for a particular {@code Service} through a service gateway).
    pub destination_type: DrgRouteRuleDestinationType,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the next hop DRG attachment responsible for reaching the network destination. <p> A value of {@code BLACKHOLE} means traffic for this route is discarded without notification.
    pub next_hop_drg_attachment_id: String,

    /// The Oracle-assigned ID of the DRG route rule.
    pub id: String,

    /// The earliest origin of a route. If a route is advertised to a DRG through an IPsec tunnel attachment, and is propagated to peered DRGs via RPC attachments, the route's provenance in the peered DRGs remains {@code IPSEC_TUNNEL}, because that is the earliest origin. <p> No routes with a provenance {@code IPSEC_TUNNEL} or {@code VIRTUAL_CIRCUIT} will be exported to IPsec tunnel or virtual circuit attachments, regardless of the attachment's export distribution.
    pub route_provenance: DrgRouteRuleRouteProvenance,
}

impl DrgRouteRule {
    /// Create a new DrgRouteRule with required fields
    pub fn new(required: DrgRouteRuleRequired) -> Self {
        Self {
            destination: required.destination,

            destination_type: required.destination_type,

            next_hop_drg_attachment_id: required.next_hop_drg_attachment_id,

            id: required.id,

            route_provenance: required.route_provenance,

            route_type: None,

            is_conflict: None,

            is_blackhole: None,

            attributes: None,
        }
    }

    /// Set destination
    pub fn set_destination(mut self, value: String) -> Self {
        self.destination = value;
        self
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: DrgRouteRuleDestinationType) -> Self {
        self.destination_type = value;
        self
    }

    /// Set next_hop_drg_attachment_id
    pub fn set_next_hop_drg_attachment_id(mut self, value: String) -> Self {
        self.next_hop_drg_attachment_id = value;
        self
    }

    /// Set route_type
    pub fn set_route_type(mut self, value: Option<DrgRouteRuleRouteType>) -> Self {
        self.route_type = value;
        self
    }

    /// Set is_conflict
    pub fn set_is_conflict(mut self, value: Option<bool>) -> Self {
        self.is_conflict = value;
        self
    }

    /// Set is_blackhole
    pub fn set_is_blackhole(mut self, value: Option<bool>) -> Self {
        self.is_blackhole = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set route_provenance
    pub fn set_route_provenance(mut self, value: DrgRouteRuleRouteProvenance) -> Self {
        self.route_provenance = value;
        self
    }

    /// Set attributes
    pub fn set_attributes(mut self, value: Option<serde_json::Value>) -> Self {
        self.attributes = value;
        self
    }

    /// Set route_type (unwraps Option)
    pub fn with_route_type(mut self, value: DrgRouteRuleRouteType) -> Self {
        self.route_type = Some(value);
        self
    }

    /// Set is_conflict (unwraps Option)
    pub fn with_is_conflict(mut self, value: bool) -> Self {
        self.is_conflict = Some(value);
        self
    }

    /// Set is_blackhole (unwraps Option)
    pub fn with_is_blackhole(mut self, value: bool) -> Self {
        self.is_blackhole = Some(value);
        self
    }

    /// Set attributes (unwraps Option)
    pub fn with_attributes(mut self, value: serde_json::Value) -> Self {
        self.attributes = Some(value);
        self
    }
}
