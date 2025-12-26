use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkRequestErrorsRequest {
    /// The ID of the asynchronous request.
    pub work_request_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For list pagination. The value of the opc-next-page response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// The field to sort by. You can provide one sort order. Default order for timestamp is descending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListWorkRequestErrorsRequestSortBy>,

    /// The sort order to use, either ascending (ASC) or descending (DESC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
}


/// Required fields for ListWorkRequestErrorsRequest
pub struct ListWorkRequestErrorsRequestRequired {
    /// The ID of the asynchronous request.
    pub work_request_id: String,
}

impl ListWorkRequestErrorsRequest {
    /// Create a new ListWorkRequestErrorsRequest with required fields
    pub fn new(required: ListWorkRequestErrorsRequestRequired) -> Self {
        Self {
            work_request_id: required.work_request_id,

            opc_request_id: None,

            page: None,

            limit: None,

            sort_by: None,

            sort_order: None,
}
    }

    /// Set work_request_id
    pub fn set_work_request_id(mut self, value: String) -> Self {
        self.work_request_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set page
    pub fn set_page(mut self, value: Option<String>) -> Self {
        self.page = value;
        self
    }

    /// Set limit
    pub fn set_limit(mut self, value: Option<i64>) -> Self {
        self.limit = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListWorkRequestErrorsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<SortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set page (unwraps Option)
    pub fn with_page(mut self, value: impl Into<String>) -> Self {
        self.page = Some(value.into());
        self
    }

    /// Set limit (unwraps Option)
    pub fn with_limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListWorkRequestErrorsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: SortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }
}


