use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For use with Oracle Cloud Infrastructure FastConnect. Each {@link VirtualCircuit} runs on one or more cross-connects or cross-connect groups. A {@code CrossConnectMapping} contains the properties for an individual cross-connect or cross-connect group associated with a given virtual circuit. <p> The mapping includes information about the cross-connect or cross-connect group, the VLAN, and the BGP peering session. <p> If you're a customer who is colocated with Oracle, that means you own both the virtual circuit and the physical connection it runs on (cross-connect or cross-connect group), so you specify all the information in the mapping. There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv4 addresses. <p> If you're a provider, then you own the physical connection that the customer's virtual circuit runs on, so you contribute information about the cross-connect or cross-connect group and VLAN. <p> Who specifies the BGP peering information in the case of customer connection via provider? If the BGP session goes from Oracle to the provider's edge router, then the provider also specifies the BGP peering information. If the BGP session instead goes from Oracle to the customer's edge router, then the customer specifies the BGP peering information. There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv4 addresses. <p> Every {@code CrossConnectMapping} must have BGP IPv4 peering addresses. BGP IPv6 peering addresses are optional. If BGP IPv6 addresses are provided, the customer can exchange IPv6 routes with Oracle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectMapping {
    /// The key for BGP MD5 authentication. Only applicable if your system requires MD5 authentication. If empty or not set (null), that means you don't use BGP MD5 authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_md5_auth_key: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect or cross-connect group for this mapping. Specified by the owner of the cross-connect or cross-connect group (the customer if the customer is colocated with Oracle, or the provider if the customer is connecting via provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_or_cross_connect_group_id: Option<String>,

    /// The BGP IPv4 address for the router on the other end of the BGP session from Oracle. Specified by the owner of that router. If the session goes from Oracle to a customer, this is the BGP IPv4 address of the customer's edge router. If the session goes from Oracle to a provider, this is the BGP IPv4 address of the provider's edge router. Must use a subnet mask from /28 to /31. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv4 addresses. <p> Example: {@code 10.0.0.18/31}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_peering_ip: Option<String>,

    /// The IPv4 address for Oracle's end of the BGP session. Must use a subnet mask from /28 to /31. If the session goes from Oracle to a customer's edge router, the customer specifies this information. If the session goes from Oracle to a provider's edge router, the provider specifies this. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv4 addresses. <p> Example: {@code 10.0.0.19/31}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_bgp_peering_ip: Option<String>,

    /// The BGP IPv6 address for the router on the other end of the BGP session from Oracle. Specified by the owner of that router. If the session goes from Oracle to a customer, this is the BGP IPv6 address of the customer's edge router. If the session goes from Oracle to a provider, this is the BGP IPv6 address of the provider's edge router. Only subnet masks from /64 up to /127 are allowed. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv6 addresses. <p> IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:db8::1/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_peering_ipv6: Option<String>,

    /// The IPv6 address for Oracle's end of the BGP session. Only subnet masks from /64 up to /127 are allowed. If the session goes from Oracle to a customer's edge router, the customer specifies this information. If the session goes from Oracle to a provider's edge router, the provider specifies this. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv6 addresses. <p> Note that IPv6 addressing is currently supported only in certain regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:db8::2/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_bgp_peering_ipv6: Option<String>,

    /// The number of the specific VLAN (on the cross-connect or cross-connect group) that is assigned to this virtual circuit. Specified by the owner of the cross-connect or cross-connect group (the customer if the customer is colocated with Oracle, or the provider if the customer is connecting via provider). <p> Example: {@code 200} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<i64>,
}

impl CrossConnectMapping {
    /// Create a new CrossConnectMapping
    pub fn new() -> Self {
        Self {
            bgp_md5_auth_key: None,

            cross_connect_or_cross_connect_group_id: None,

            customer_bgp_peering_ip: None,

            oracle_bgp_peering_ip: None,

            customer_bgp_peering_ipv6: None,

            oracle_bgp_peering_ipv6: None,

            vlan: None,
        }
    }

    /// Set bgp_md5_auth_key
    pub fn set_bgp_md5_auth_key(mut self, value: Option<String>) -> Self {
        self.bgp_md5_auth_key = value;
        self
    }

    /// Set cross_connect_or_cross_connect_group_id
    pub fn set_cross_connect_or_cross_connect_group_id(mut self, value: Option<String>) -> Self {
        self.cross_connect_or_cross_connect_group_id = value;
        self
    }

    /// Set customer_bgp_peering_ip
    pub fn set_customer_bgp_peering_ip(mut self, value: Option<String>) -> Self {
        self.customer_bgp_peering_ip = value;
        self
    }

    /// Set oracle_bgp_peering_ip
    pub fn set_oracle_bgp_peering_ip(mut self, value: Option<String>) -> Self {
        self.oracle_bgp_peering_ip = value;
        self
    }

    /// Set customer_bgp_peering_ipv6
    pub fn set_customer_bgp_peering_ipv6(mut self, value: Option<String>) -> Self {
        self.customer_bgp_peering_ipv6 = value;
        self
    }

    /// Set oracle_bgp_peering_ipv6
    pub fn set_oracle_bgp_peering_ipv6(mut self, value: Option<String>) -> Self {
        self.oracle_bgp_peering_ipv6 = value;
        self
    }

    /// Set vlan
    pub fn set_vlan(mut self, value: Option<i64>) -> Self {
        self.vlan = value;
        self
    }

    /// Set bgp_md5_auth_key (unwraps Option)
    pub fn with_bgp_md5_auth_key(mut self, value: impl Into<String>) -> Self {
        self.bgp_md5_auth_key = Some(value.into());
        self
    }

    /// Set cross_connect_or_cross_connect_group_id (unwraps Option)
    pub fn with_cross_connect_or_cross_connect_group_id(
        mut self,
        value: impl Into<String>,
    ) -> Self {
        self.cross_connect_or_cross_connect_group_id = Some(value.into());
        self
    }

    /// Set customer_bgp_peering_ip (unwraps Option)
    pub fn with_customer_bgp_peering_ip(mut self, value: impl Into<String>) -> Self {
        self.customer_bgp_peering_ip = Some(value.into());
        self
    }

    /// Set oracle_bgp_peering_ip (unwraps Option)
    pub fn with_oracle_bgp_peering_ip(mut self, value: impl Into<String>) -> Self {
        self.oracle_bgp_peering_ip = Some(value.into());
        self
    }

    /// Set customer_bgp_peering_ipv6 (unwraps Option)
    pub fn with_customer_bgp_peering_ipv6(mut self, value: impl Into<String>) -> Self {
        self.customer_bgp_peering_ipv6 = Some(value.into());
        self
    }

    /// Set oracle_bgp_peering_ipv6 (unwraps Option)
    pub fn with_oracle_bgp_peering_ipv6(mut self, value: impl Into<String>) -> Self {
        self.oracle_bgp_peering_ipv6 = Some(value.into());
        self
    }

    /// Set vlan (unwraps Option)
    pub fn with_vlan(mut self, value: i64) -> Self {
        self.vlan = Some(value);
        self
    }
}

impl Default for CrossConnectMapping {
    fn default() -> Self {
        Self::new()
    }
}
