use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIPSecConnectionTunnelRoutesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub tunnel_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Specifies the advertiser of the routes. If set to {@code ORACLE}, this returns only the routes advertised by Oracle. When set to {@code CUSTOMER}, this returns only the routes advertised by the CPE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advertiser: Option<String>,
}

/// Required fields for ListIPSecConnectionTunnelRoutesRequest
pub struct ListIPSecConnectionTunnelRoutesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPSec connection.
    pub ipsc_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub tunnel_id: String,
}

impl ListIPSecConnectionTunnelRoutesRequest {
    /// Create a new ListIPSecConnectionTunnelRoutesRequest with required fields
    pub fn new(required: ListIPSecConnectionTunnelRoutesRequestRequired) -> Self {
        Self {
            ipsc_id: required.ipsc_id,

            tunnel_id: required.tunnel_id,

            limit: None,

            page: None,

            advertiser: None,
        }
    }

    /// Set ipsc_id
    pub fn set_ipsc_id(mut self, value: String) -> Self {
        self.ipsc_id = value;
        self
    }

    /// Set tunnel_id
    pub fn set_tunnel_id(mut self, value: String) -> Self {
        self.tunnel_id = value;
        self
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

    /// Set advertiser
    pub fn set_advertiser(mut self, value: Option<String>) -> Self {
        self.advertiser = value;
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

    /// Set advertiser (unwraps Option)
    pub fn with_advertiser(mut self, value: impl Into<String>) -> Self {
        self.advertiser = Some(value.into());
        self
    }
}
