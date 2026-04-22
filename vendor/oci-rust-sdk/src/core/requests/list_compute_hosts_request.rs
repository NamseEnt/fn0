use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListComputeHostsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute host network resoruce. - Customer-unique HPC island ID - Customer-unique network block ID - Customer-unique local block ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_resource_id: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListComputeHostsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListComputeHostsRequestSortOrder>,

    /// A filter to return only ComputeHostSummary resources that match the given Compute Host lifecycle State OCID exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_lifecycle_state: Option<String>,

    /// A filter to return only ComputeHostSummary resources that match the given Compute Host health State OCID exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_health: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute host group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_group_id: Option<String>,

    /// When set to true, all the compartments in the tenancy are traversed and the hosts in the specified tenancy and its compartments are fetched. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_in_subtree: Option<bool>,
}

/// Required fields for ListComputeHostsRequest
pub struct ListComputeHostsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListComputeHostsRequest {
    /// Create a new ListComputeHostsRequest with required fields
    pub fn new(required: ListComputeHostsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            opc_request_id: None,

            availability_domain: None,

            display_name: None,

            network_resource_id: None,

            limit: None,

            page: None,

            sort_by: None,

            sort_order: None,

            compute_host_lifecycle_state: None,

            compute_host_health: None,

            compute_host_group_id: None,

            compute_host_in_subtree: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set network_resource_id
    pub fn set_network_resource_id(mut self, value: Option<String>) -> Self {
        self.network_resource_id = value;
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

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListComputeHostsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListComputeHostsRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set compute_host_lifecycle_state
    pub fn set_compute_host_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.compute_host_lifecycle_state = value;
        self
    }

    /// Set compute_host_health
    pub fn set_compute_host_health(mut self, value: Option<String>) -> Self {
        self.compute_host_health = value;
        self
    }

    /// Set compute_host_group_id
    pub fn set_compute_host_group_id(mut self, value: Option<String>) -> Self {
        self.compute_host_group_id = value;
        self
    }

    /// Set compute_host_in_subtree
    pub fn set_compute_host_in_subtree(mut self, value: Option<bool>) -> Self {
        self.compute_host_in_subtree = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set network_resource_id (unwraps Option)
    pub fn with_network_resource_id(mut self, value: impl Into<String>) -> Self {
        self.network_resource_id = Some(value.into());
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

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListComputeHostsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListComputeHostsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set compute_host_lifecycle_state (unwraps Option)
    pub fn with_compute_host_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.compute_host_lifecycle_state = Some(value.into());
        self
    }

    /// Set compute_host_health (unwraps Option)
    pub fn with_compute_host_health(mut self, value: impl Into<String>) -> Self {
        self.compute_host_health = Some(value.into());
        self
    }

    /// Set compute_host_group_id (unwraps Option)
    pub fn with_compute_host_group_id(mut self, value: impl Into<String>) -> Self {
        self.compute_host_group_id = Some(value.into());
        self
    }

    /// Set compute_host_in_subtree (unwraps Option)
    pub fn with_compute_host_in_subtree(mut self, value: bool) -> Self {
        self.compute_host_in_subtree = Some(value);
        self
    }
}
