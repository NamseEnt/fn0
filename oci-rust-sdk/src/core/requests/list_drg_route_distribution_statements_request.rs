use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDrgRouteDistributionStatementsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The field to sort by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListDrgRouteDistributionStatementsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListDrgRouteDistributionStatementsRequestSortOrder>,
}

/// Required fields for ListDrgRouteDistributionStatementsRequest
pub struct ListDrgRouteDistributionStatementsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,
}

impl ListDrgRouteDistributionStatementsRequest {
    /// Create a new ListDrgRouteDistributionStatementsRequest with required fields
    pub fn new(required: ListDrgRouteDistributionStatementsRequestRequired) -> Self {
        Self {
            drg_route_distribution_id: required.drg_route_distribution_id,

            limit: None,

            page: None,

            sort_by: None,

            sort_order: None,
        }
    }

    /// Set drg_route_distribution_id
    pub fn set_drg_route_distribution_id(mut self, value: String) -> Self {
        self.drg_route_distribution_id = value;
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
    pub fn set_sort_by(
        mut self,
        value: Option<ListDrgRouteDistributionStatementsRequestSortBy>,
    ) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(
        mut self,
        value: Option<ListDrgRouteDistributionStatementsRequestSortOrder>,
    ) -> Self {
        self.sort_order = value;
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
    pub fn with_sort_by(mut self, value: ListDrgRouteDistributionStatementsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(
        mut self,
        value: ListDrgRouteDistributionStatementsRequestSortOrder,
    ) -> Self {
        self.sort_order = Some(value);
        self
    }
}
