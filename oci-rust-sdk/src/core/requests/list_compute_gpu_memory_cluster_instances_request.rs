use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeGpuMemoryClusterInstancesRequest {
    /// The OCID of the compute GPU memory cluster.
    pub compute_gpu_memory_cluster_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListComputeGpuMemoryClusterInstancesRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListComputeGpuMemoryClusterInstancesRequestSortOrder>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}


/// Required fields for ListComputeGpuMemoryClusterInstancesRequest
pub struct ListComputeGpuMemoryClusterInstancesRequestRequired {
    /// The OCID of the compute GPU memory cluster.
    pub compute_gpu_memory_cluster_id: String,
}

impl ListComputeGpuMemoryClusterInstancesRequest {
    /// Create a new ListComputeGpuMemoryClusterInstancesRequest with required fields
    pub fn new(required: ListComputeGpuMemoryClusterInstancesRequestRequired) -> Self {
        Self {
            compute_gpu_memory_cluster_id: required.compute_gpu_memory_cluster_id,

            opc_request_id: None,

            page: None,

            sort_by: None,

            sort_order: None,

            limit: None,
}
    }

    /// Set compute_gpu_memory_cluster_id
    pub fn set_compute_gpu_memory_cluster_id(mut self, value: String) -> Self {
        self.compute_gpu_memory_cluster_id = value;
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

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListComputeGpuMemoryClusterInstancesRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListComputeGpuMemoryClusterInstancesRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set limit
    pub fn set_limit(mut self, value: Option<i64>) -> Self {
        self.limit = value;
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

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListComputeGpuMemoryClusterInstancesRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListComputeGpuMemoryClusterInstancesRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set limit (unwraps Option)
    pub fn with_limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }
}


