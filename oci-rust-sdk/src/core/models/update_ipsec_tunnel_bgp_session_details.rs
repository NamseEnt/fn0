use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecTunnelBgpSessionDetails {
    /// The IP address for the Oracle end of the inside tunnel interface. <p> If the tunnel's {@code routing} attribute is set to {@code BGP} (see {@link #updateIPSecConnectionTunnelDetails(UpdateIPSecConnectionTunnelDetailsRequest) updateIPSecConnectionTunnelDetails}), this IP address is used for the tunnel's BGP session. <p> If {@code routing} is instead set to {@code STATIC}, you can set this IP address to troubleshoot or monitor the tunnel. <p> The value must be a /30 or /31. <p> If you are switching the tunnel from using BGP dynamic routing to static routing and want to remove the value for {@code oracleInterfaceIp}, you can set the value to an empty string. <p> Example: {@code 10.0.0.4/31}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_interface_ip: Option<String>,

    /// The IP address for the CPE end of the inside tunnel interface. <p> If the tunnel's {@code routing} attribute is set to {@code BGP} (see {@link #updateIPSecConnectionTunnelDetails(UpdateIPSecConnectionTunnelDetailsRequest) updateIPSecConnectionTunnelDetails}), this IP address is used for the tunnel's BGP session. <p> If {@code routing} is instead set to {@code STATIC}, you can set this IP address to troubleshoot or monitor the tunnel. <p> The value must be a /30 or /31. <p> If you are switching the tunnel from using BGP dynamic routing to static routing and want to remove the value for {@code customerInterfaceIp}, you can set the value to an empty string. <p> Example: {@code 10.0.0.5/31}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_interface_ip: Option<String>,

    /// The IPv6 address for the Oracle end of the inside tunnel interface. This IP address is optional. <p> If the tunnel's {@code routing} attribute is set to {@code BGP} (see {@link IPSecConnectionTunnel}), this IP address is used for the tunnel's BGP session. <p> If {@code routing} is instead set to {@code STATIC}, you can set this IP address to troubleshoot or monitor the tunnel. <p> Only subnet masks from /64 up to /127 are allowed. <p> Example: {@code 2001:db8::1/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_interface_ipv6: Option<String>,

    /// The IPv6 address for the CPE end of the inside tunnel interface. This IP address is optional. <p> If the tunnel's {@code routing} attribute is set to {@code BGP} (see {@link IPSecConnectionTunnel}), this IP address is used for the tunnel's BGP session. <p> If {@code routing} is instead set to {@code STATIC}, you can set this IP address to troubleshoot or monitor the tunnel. <p> Only subnet masks from /64 up to /127 are allowed. <p> Example: {@code 2001:db8::1/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_interface_ipv6: Option<String>,

    /// The BGP ASN of the network on the CPE end of the BGP session. Can be a 2-byte or 4-byte ASN. Uses \"asplain\" format. <p> If you are switching the tunnel from using BGP dynamic routing to static routing, the {@code customerBgpAsn} must be null. <p> Example: {@code 12345} (2-byte) or {@code 1587232876} (4-byte)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_asn: Option<String>,
}

impl UpdateIPSecTunnelBgpSessionDetails {
    /// Create a new UpdateIPSecTunnelBgpSessionDetails
    pub fn new() -> Self {
        Self {
            oracle_interface_ip: None,

            customer_interface_ip: None,

            oracle_interface_ipv6: None,

            customer_interface_ipv6: None,

            customer_bgp_asn: None,
        }
    }

    /// Set oracle_interface_ip
    pub fn set_oracle_interface_ip(mut self, value: Option<String>) -> Self {
        self.oracle_interface_ip = value;
        self
    }

    /// Set customer_interface_ip
    pub fn set_customer_interface_ip(mut self, value: Option<String>) -> Self {
        self.customer_interface_ip = value;
        self
    }

    /// Set oracle_interface_ipv6
    pub fn set_oracle_interface_ipv6(mut self, value: Option<String>) -> Self {
        self.oracle_interface_ipv6 = value;
        self
    }

    /// Set customer_interface_ipv6
    pub fn set_customer_interface_ipv6(mut self, value: Option<String>) -> Self {
        self.customer_interface_ipv6 = value;
        self
    }

    /// Set customer_bgp_asn
    pub fn set_customer_bgp_asn(mut self, value: Option<String>) -> Self {
        self.customer_bgp_asn = value;
        self
    }

    /// Set oracle_interface_ip (unwraps Option)
    pub fn with_oracle_interface_ip(mut self, value: impl Into<String>) -> Self {
        self.oracle_interface_ip = Some(value.into());
        self
    }

    /// Set customer_interface_ip (unwraps Option)
    pub fn with_customer_interface_ip(mut self, value: impl Into<String>) -> Self {
        self.customer_interface_ip = Some(value.into());
        self
    }

    /// Set oracle_interface_ipv6 (unwraps Option)
    pub fn with_oracle_interface_ipv6(mut self, value: impl Into<String>) -> Self {
        self.oracle_interface_ipv6 = Some(value.into());
        self
    }

    /// Set customer_interface_ipv6 (unwraps Option)
    pub fn with_customer_interface_ipv6(mut self, value: impl Into<String>) -> Self {
        self.customer_interface_ipv6 = Some(value.into());
        self
    }

    /// Set customer_bgp_asn (unwraps Option)
    pub fn with_customer_bgp_asn(mut self, value: impl Into<String>) -> Self {
        self.customer_bgp_asn = Some(value.into());
        self
    }
}

impl Default for UpdateIPSecTunnelBgpSessionDetails {
    fn default() -> Self {
        Self::new()
    }
}
