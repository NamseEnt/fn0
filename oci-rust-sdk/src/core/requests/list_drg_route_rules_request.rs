use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDrgRouteRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Static routes are specified through the DRG route table API. Dynamic routes are learned by the DRG from the DRG attachments through various routing protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_type: Option<ListDrgRouteRulesRequestRouteType>,
}


/// Required fields for ListDrgRouteRulesRequest
pub struct ListDrgRouteRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,
}

impl ListDrgRouteRulesRequest {
    /// Create a new ListDrgRouteRulesRequest with required fields
    pub fn new(required: ListDrgRouteRulesRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            limit: None,

            page: None,

            route_type: None,
}
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
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

    /// Set route_type
    pub fn set_route_type(mut self, value: Option<ListDrgRouteRulesRequestRouteType>) -> Self {
        self.route_type = value;
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

    /// Set route_type (unwraps Option)
    pub fn with_route_type(mut self, value: ListDrgRouteRulesRequestRouteType) -> Self {
        self.route_type = Some(value);
        self
    }
}


