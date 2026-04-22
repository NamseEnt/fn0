use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpv6sRequest {
    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// An IP address. This could be either IPv4 or IPv6, depending on the resource. Example: {@code 10.0.3.3}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// The OCID of the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,

    /// State of the IP address. If an IP address is assigned to a VNIC it is ASSIGNED otherwise AVAILABLE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_state: Option<String>,

    /// Lifetime of the IP address. There are two types of IPs: - Ephemeral - Reserved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

impl ListIpv6sRequest {
    /// Create a new ListIpv6sRequest
    pub fn new() -> Self {
        Self {
            limit: None,

            page: None,

            ip_address: None,

            subnet_id: None,

            vnic_id: None,

            ip_state: None,

            lifetime: None,

            opc_request_id: None,
        }
    }

    /// Set limit
    pub fn set_limit(mut self, value: Option<i64>) -> Self {
        self.limit = value;
        self
    }

    /// Set page
    pub fn set_page(mut self, value: Option<String>) -> Self {
        self.page = value;
        self
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set ip_state
    pub fn set_ip_state(mut self, value: Option<String>) -> Self {
        self.ip_state = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<String>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set limit (unwraps Option)
    pub fn with_limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    /// Set page (unwraps Option)
    pub fn with_page(mut self, value: impl Into<String>) -> Self {
        self.page = Some(value.into());
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }

    /// Set ip_state (unwraps Option)
    pub fn with_ip_state(mut self, value: impl Into<String>) -> Self {
        self.ip_state = Some(value.into());
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: impl Into<String>) -> Self {
        self.lifetime = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}

impl Default for ListIpv6sRequest {
    fn default() -> Self {
        Self::new()
    }
}
