use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkRequestsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment in which to list resources.
    pub compartment_id: String,

    /// The ID of the asynchronous work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_id: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For list pagination. The value of the opc-next-page response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// A filter to return resources with a lifecycleState that matches the given OperationStatus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OperationStatus>,

    /// The field to sort by. You can provide one sort order. Default order for timeAccepted is descending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListWorkRequestsRequestSortBy>,

    /// The sort order to use, either ascending (ASC) or descending (DESC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,

    /// The OCID of the resource affected by the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}


/// Required fields for ListWorkRequestsRequest
pub struct ListWorkRequestsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment in which to list resources.
    pub compartment_id: String,
}

impl ListWorkRequestsRequest {
    /// Create a new ListWorkRequestsRequest with required fields
    pub fn new(required: ListWorkRequestsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            work_request_id: None,

            opc_request_id: None,

            page: None,

            limit: None,

            availability_domain: None,

            status: None,

            sort_by: None,

            sort_order: None,

            resource_id: None,
}
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set work_request_id
    pub fn set_work_request_id(mut self, value: Option<String>) -> Self {
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

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: Option<OperationStatus>) -> Self {
        self.status = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListWorkRequestsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<SortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set resource_id
    pub fn set_resource_id(mut self, value: Option<String>) -> Self {
        self.resource_id = value;
        self
    }

    /// Set work_request_id (unwraps Option)
    pub fn with_work_request_id(mut self, value: impl Into<String>) -> Self {
        self.work_request_id = Some(value.into());
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

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set status (unwraps Option)
    pub fn with_status(mut self, value: OperationStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListWorkRequestsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: SortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set resource_id (unwraps Option)
    pub fn with_resource_id(mut self, value: impl Into<String>) -> Self {
        self.resource_id = Some(value.into());
        self
    }
}


