use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListImagesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The image's operating system. <p> Example: {@code Oracle Linux}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,

    /// The image's operating system version. <p> Example: {@code 7.2}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system_version: Option<String>,

    /// Shape name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListImagesRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListImagesRequestSortOrder>,

    /// A filter to only return resources that match the given lifecycle state. The state value is case-insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
}


/// Required fields for ListImagesRequest
pub struct ListImagesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListImagesRequest {
    /// Create a new ListImagesRequest with required fields
    pub fn new(required: ListImagesRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: None,

            operating_system: None,

            operating_system_version: None,

            shape: None,

            limit: None,

            page: None,

            sort_by: None,

            sort_order: None,

            lifecycle_state: None,
}
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set operating_system
    pub fn set_operating_system(mut self, value: Option<String>) -> Self {
        self.operating_system = value;
        self
    }

    /// Set operating_system_version
    pub fn set_operating_system_version(mut self, value: Option<String>) -> Self {
        self.operating_system_version = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: Option<String>) -> Self {
        self.shape = value;
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
    pub fn set_sort_by(mut self, value: Option<ListImagesRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListImagesRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set operating_system (unwraps Option)
    pub fn with_operating_system(mut self, value: impl Into<String>) -> Self {
        self.operating_system = Some(value.into());
        self
    }

    /// Set operating_system_version (unwraps Option)
    pub fn with_operating_system_version(mut self, value: impl Into<String>) -> Self {
        self.operating_system_version = Some(value.into());
        self
    }

    /// Set shape (unwraps Option)
    pub fn with_shape(mut self, value: impl Into<String>) -> Self {
        self.shape = Some(value.into());
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
    pub fn with_sort_by(mut self, value: ListImagesRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListImagesRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }
}


