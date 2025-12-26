use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVolumeGroupReplicasRequest {
    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListVolumeGroupReplicasRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListVolumeGroupReplicasRequestSortOrder>,

    /// A filter to only return resources that match the given lifecycle state. The state value is case-insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
}


/// Required fields for ListVolumeGroupReplicasRequest
pub struct ListVolumeGroupReplicasRequestRequired {
    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListVolumeGroupReplicasRequest {
    /// Create a new ListVolumeGroupReplicasRequest with required fields
    pub fn new(required: ListVolumeGroupReplicasRequestRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            limit: None,

            page: None,

            display_name: None,

            sort_by: None,

            sort_order: None,

            lifecycle_state: None,
}
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListVolumeGroupReplicasRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListVolumeGroupReplicasRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
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

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListVolumeGroupReplicasRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListVolumeGroupReplicasRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }
}


