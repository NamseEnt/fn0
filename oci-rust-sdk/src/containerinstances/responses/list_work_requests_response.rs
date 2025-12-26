use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkRequestsResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.WorkRequestSummaryCollection instance.
    pub work_request_summary_collection: WorkRequestSummaryCollection,
}


/// Required fields for ListWorkRequestsResponse
pub struct ListWorkRequestsResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.WorkRequestSummaryCollection instance.
    pub work_request_summary_collection: WorkRequestSummaryCollection,
}

impl ListWorkRequestsResponse {
    /// Create a new ListWorkRequestsResponse with required fields
    pub fn new(required: ListWorkRequestsResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            opc_next_page: required.opc_next_page,

            work_request_summary_collection: required.work_request_summary_collection,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: String) -> Self {
        self.opc_next_page = value;
        self
    }

    /// Set work_request_summary_collection
    pub fn set_work_request_summary_collection(mut self, value: WorkRequestSummaryCollection) -> Self {
        self.work_request_summary_collection = value;
        self
    }
}


