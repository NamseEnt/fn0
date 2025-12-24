use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For use with Oracle Cloud Infrastructure FastConnect. Each {@link VirtualCircuit} runs on one or more cross-connects or cross-connect groups. A {@code CrossConnectMappingDetails} contains the properties for an individual cross-connect or cross-connect group associated with a given virtual circuit. <p> The details includes information about the cross-connect or cross-connect group, the VLAN, and the BGP peering session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectMappingDetails {
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

    /// The BGP IPv6 address for the router on the other end of the BGP session from Oracle. Specified by the owner of that router. If the session goes from Oracle to a customer, this is the BGP IPv6 address of the customer's edge router. If the session goes from Oracle to a provider, this is the BGP IPv6 address of the provider's edge router. Only subnet masks from /64 up to /127 are allowed. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv6 addresses. <p> Example: {@code 2001:db8::1/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_bgp_peering_ipv6: Option<String>,

    /// The IPv6 address for Oracle's end of the BGP session. Only subnet masks from /64 up to /127 are allowed. If the session goes from Oracle to a customer's edge router, the customer specifies this information. If the session goes from Oracle to a provider's edge router, the provider specifies this. <p> There's one exception: for a public virtual circuit, Oracle specifies the BGP IPv6 addresses. <p> Example: {@code 2001:db8::2/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_bgp_peering_ipv6: Option<String>,

    /// The number of the specific VLAN (on the cross-connect or cross-connect group) that is assigned to this virtual circuit. Specified by the owner of the cross-connect or cross-connect group (the customer if the customer is colocated with Oracle, or the provider if the customer is connecting via provider). <p> Example: {@code 200} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<i64>,

    /// The state of the Ipv4 BGP session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_bgp_status: Option<CrossConnectMappingDetailsIpv4BgpStatus>,

    /// The state of the Ipv6 BGP session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_bgp_status: Option<CrossConnectMappingDetailsIpv6BgpStatus>,

    /// The FastConnect device that terminates the logical connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oci_logical_device_name: Option<String>,
}

impl CrossConnectMappingDetails {
    /// Create a new CrossConnectMappingDetails
    pub fn new() -> Self {
        Self {
            bgp_md5_auth_key: None,

            cross_connect_or_cross_connect_group_id: None,

            customer_bgp_peering_ip: None,

            oracle_bgp_peering_ip: None,

            customer_bgp_peering_ipv6: None,

            oracle_bgp_peering_ipv6: None,

            vlan: None,

            ipv4_bgp_status: None,

            ipv6_bgp_status: None,

            oci_logical_device_name: None,
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

    /// Set ipv4_bgp_status
    pub fn set_ipv4_bgp_status(
        mut self,
        value: Option<CrossConnectMappingDetailsIpv4BgpStatus>,
    ) -> Self {
        self.ipv4_bgp_status = value;
        self
    }

    /// Set ipv6_bgp_status
    pub fn set_ipv6_bgp_status(
        mut self,
        value: Option<CrossConnectMappingDetailsIpv6BgpStatus>,
    ) -> Self {
        self.ipv6_bgp_status = value;
        self
    }

    /// Set oci_logical_device_name
    pub fn set_oci_logical_device_name(mut self, value: Option<String>) -> Self {
        self.oci_logical_device_name = value;
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

    /// Set ipv4_bgp_status (unwraps Option)
    pub fn with_ipv4_bgp_status(mut self, value: CrossConnectMappingDetailsIpv4BgpStatus) -> Self {
        self.ipv4_bgp_status = Some(value);
        self
    }

    /// Set ipv6_bgp_status (unwraps Option)
    pub fn with_ipv6_bgp_status(mut self, value: CrossConnectMappingDetailsIpv6BgpStatus) -> Self {
        self.ipv6_bgp_status = Some(value);
        self
    }

    /// Set oci_logical_device_name (unwraps Option)
    pub fn with_oci_logical_device_name(mut self, value: impl Into<String>) -> Self {
        self.oci_logical_device_name = Some(value.into());
        self
    }
}

impl Default for CrossConnectMappingDetails {
    fn default() -> Self {
        Self::new()
    }
}
