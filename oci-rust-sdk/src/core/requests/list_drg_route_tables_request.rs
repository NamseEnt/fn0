use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDrgRouteTablesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

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
    pub sort_by: Option<ListDrgRouteTablesRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListDrgRouteTablesRequestSortOrder>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the import route distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_drg_route_distribution_id: Option<String>,

    /// A filter that only returns matches for the specified lifecycle state. The value is case insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
}


/// Required fields for ListDrgRouteTablesRequest
pub struct ListDrgRouteTablesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,
}

impl ListDrgRouteTablesRequest {
    /// Create a new ListDrgRouteTablesRequest with required fields
    pub fn new(required: ListDrgRouteTablesRequestRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            limit: None,

            page: None,

            display_name: None,

            sort_by: None,

            sort_order: None,

            import_drg_route_distribution_id: None,

            lifecycle_state: None,
}
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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
    pub fn set_sort_by(mut self, value: Option<ListDrgRouteTablesRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListDrgRouteTablesRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set import_drg_route_distribution_id
    pub fn set_import_drg_route_distribution_id(mut self, value: Option<String>) -> Self {
        self.import_drg_route_distribution_id = value;
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
    pub fn with_sort_by(mut self, value: ListDrgRouteTablesRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListDrgRouteTablesRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set import_drg_route_distribution_id (unwraps Option)
    pub fn with_import_drg_route_distribution_id(mut self, value: impl Into<String>) -> Self {
        self.import_drg_route_distribution_id = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }
}


