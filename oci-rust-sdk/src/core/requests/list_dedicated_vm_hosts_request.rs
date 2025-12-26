use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDedicatedVmHostsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// A filter to only return resources that match the given lifecycle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ListDedicatedVmHostsRequestLifecycleState>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The name for the instance's shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape_name: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListDedicatedVmHostsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListDedicatedVmHostsRequestSortOrder>,

    /// The remaining memory of the dedicated VM host, in GBs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_memory_in_gbs_greater_than_or_equal_to: Option<i64>,

    /// The available OCPUs of the dedicated VM host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_ocpus_greater_than_or_equal_to: Option<i64>,

    /// A filter to return only confidential Dedicated VM hosts (DVMH) or confidential VM instances on DVMH.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,
}


/// Required fields for ListDedicatedVmHostsRequest
pub struct ListDedicatedVmHostsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListDedicatedVmHostsRequest {
    /// Create a new ListDedicatedVmHostsRequest with required fields
    pub fn new(required: ListDedicatedVmHostsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            availability_domain: None,

            lifecycle_state: None,

            display_name: None,

            instance_shape_name: None,

            limit: None,

            page: None,

            opc_request_id: None,

            sort_by: None,

            sort_order: None,

            remaining_memory_in_gbs_greater_than_or_equal_to: None,

            remaining_ocpus_greater_than_or_equal_to: None,

            is_memory_encryption_enabled: None,
}
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<ListDedicatedVmHostsRequestLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set instance_shape_name
    pub fn set_instance_shape_name(mut self, value: Option<String>) -> Self {
        self.instance_shape_name = value;
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

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListDedicatedVmHostsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListDedicatedVmHostsRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set remaining_memory_in_gbs_greater_than_or_equal_to
    pub fn set_remaining_memory_in_gbs_greater_than_or_equal_to(mut self, value: Option<i64>) -> Self {
        self.remaining_memory_in_gbs_greater_than_or_equal_to = value;
        self
    }

    /// Set remaining_ocpus_greater_than_or_equal_to
    pub fn set_remaining_ocpus_greater_than_or_equal_to(mut self, value: Option<i64>) -> Self {
        self.remaining_ocpus_greater_than_or_equal_to = value;
        self
    }

    /// Set is_memory_encryption_enabled
    pub fn set_is_memory_encryption_enabled(mut self, value: Option<bool>) -> Self {
        self.is_memory_encryption_enabled = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: ListDedicatedVmHostsRequestLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set instance_shape_name (unwraps Option)
    pub fn with_instance_shape_name(mut self, value: impl Into<String>) -> Self {
        self.instance_shape_name = Some(value.into());
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

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListDedicatedVmHostsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListDedicatedVmHostsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set remaining_memory_in_gbs_greater_than_or_equal_to (unwraps Option)
    pub fn with_remaining_memory_in_gbs_greater_than_or_equal_to(mut self, value: i64) -> Self {
        self.remaining_memory_in_gbs_greater_than_or_equal_to = Some(value);
        self
    }

    /// Set remaining_ocpus_greater_than_or_equal_to (unwraps Option)
    pub fn with_remaining_ocpus_greater_than_or_equal_to(mut self, value: i64) -> Self {
        self.remaining_ocpus_greater_than_or_equal_to = Some(value);
        self
    }

    /// Set is_memory_encryption_enabled (unwraps Option)
    pub fn with_is_memory_encryption_enabled(mut self, value: bool) -> Self {
        self.is_memory_encryption_enabled = Some(value);
        self
    }
}


