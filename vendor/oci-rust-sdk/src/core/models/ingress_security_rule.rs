use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A rule for allowing inbound IP packets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngressSecurityRule {
    /// The transport protocol. Specify either {@code all} or an IPv4 protocol number as defined in [Protocol Numbers](http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml). Options are supported only for ICMP (\"1\"), TCP (\"6\"), UDP (\"17\"), and ICMPv6 (\"58\").
    pub protocol: String,

    /// Conceptually, this is the range of IP addresses that a packet coming into the instance can come from. <p> Allowed values: <p> IP address range in CIDR notation. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}. IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> The {@code cidrBlock} value for a {@link Service}, if you're setting up a security list rule for traffic coming from a particular {@code Service} through a service gateway. For example: {@code oci-phx-objectstorage}.
    pub source: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icmp_options: Option<IcmpOptions>,

    /// A stateless rule allows traffic in one direction. Remember to add a corresponding stateless rule in the other direction if you need to support bidirectional traffic. For example, if ingress traffic allows TCP destination port 80, there should be an egress rule to allow TCP source port 80. Defaults to false, which means the rule is stateful and a corresponding rule is not necessary for bidirectional traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_stateless: Option<bool>,

    /// Type of source for the rule. The default is {@code CIDR_BLOCK}. <p> {@code CIDR_BLOCK}: If the rule's {@code source} is an IP address range in CIDR notation. <p> {@code SERVICE_CIDR_BLOCK}: If the rule's {@code source} is the {@code cidrBlock} value for a {@link Service} (the rule is for traffic coming from a particular {@code Service} through a service gateway).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<IngressSecurityRuleSourceType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_options: Option<TcpOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_options: Option<UdpOptions>,

    /// An optional description of your choice for the rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Required fields for IngressSecurityRule
pub struct IngressSecurityRuleRequired {
    /// The transport protocol. Specify either {@code all} or an IPv4 protocol number as defined in [Protocol Numbers](http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml). Options are supported only for ICMP (\"1\"), TCP (\"6\"), UDP (\"17\"), and ICMPv6 (\"58\").
    pub protocol: String,

    /// Conceptually, this is the range of IP addresses that a packet coming into the instance can come from. <p> Allowed values: <p> IP address range in CIDR notation. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}. IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> The {@code cidrBlock} value for a {@link Service}, if you're setting up a security list rule for traffic coming from a particular {@code Service} through a service gateway. For example: {@code oci-phx-objectstorage}.
    pub source: String,
}

impl IngressSecurityRule {
    /// Create a new IngressSecurityRule with required fields
    pub fn new(required: IngressSecurityRuleRequired) -> Self {
        Self {
            protocol: required.protocol,

            source: required.source,

            icmp_options: None,

            is_stateless: None,

            source_type: None,

            tcp_options: None,

            udp_options: None,

            description: None,
        }
    }

    /// Set icmp_options
    pub fn set_icmp_options(mut self, value: Option<IcmpOptions>) -> Self {
        self.icmp_options = value;
        self
    }

    /// Set is_stateless
    pub fn set_is_stateless(mut self, value: Option<bool>) -> Self {
        self.is_stateless = value;
        self
    }

    /// Set protocol
    pub fn set_protocol(mut self, value: String) -> Self {
        self.protocol = value;
        self
    }

    /// Set source
    pub fn set_source(mut self, value: String) -> Self {
        self.source = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: Option<IngressSecurityRuleSourceType>) -> Self {
        self.source_type = value;
        self
    }

    /// Set tcp_options
    pub fn set_tcp_options(mut self, value: Option<TcpOptions>) -> Self {
        self.tcp_options = value;
        self
    }

    /// Set udp_options
    pub fn set_udp_options(mut self, value: Option<UdpOptions>) -> Self {
        self.udp_options = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set icmp_options (unwraps Option)
    pub fn with_icmp_options(mut self, value: IcmpOptions) -> Self {
        self.icmp_options = Some(value);
        self
    }

    /// Set is_stateless (unwraps Option)
    pub fn with_is_stateless(mut self, value: bool) -> Self {
        self.is_stateless = Some(value);
        self
    }

    /// Set source_type (unwraps Option)
    pub fn with_source_type(mut self, value: IngressSecurityRuleSourceType) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set tcp_options (unwraps Option)
    pub fn with_tcp_options(mut self, value: TcpOptions) -> Self {
        self.tcp_options = Some(value);
        self
    }

    /// Set udp_options (unwraps Option)
    pub fn with_udp_options(mut self, value: UdpOptions) -> Self {
        self.udp_options = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}
