use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIPSecConnectionsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_id: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

/// Required fields for ListIPSecConnectionsRequest
pub struct ListIPSecConnectionsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListIPSecConnectionsRequest {
    /// Create a new ListIPSecConnectionsRequest with required fields
    pub fn new(required: ListIPSecConnectionsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            drg_id: None,

            cpe_id: None,

            limit: None,

            page: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: Option<String>) -> Self {
        self.drg_id = value;
        self
    }

    /// Set cpe_id
    pub fn set_cpe_id(mut self, value: Option<String>) -> Self {
        self.cpe_id = value;
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

    /// Set drg_id (unwraps Option)
    pub fn with_drg_id(mut self, value: impl Into<String>) -> Self {
        self.drg_id = Some(value.into());
        self
    }

    /// Set cpe_id (unwraps Option)
    pub fn with_cpe_id(mut self, value: impl Into<String>) -> Self {
        self.cpe_id = Some(value.into());
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
}
