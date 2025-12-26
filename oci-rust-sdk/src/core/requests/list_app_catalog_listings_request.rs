use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAppCatalogListingsRequest {
    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListAppCatalogListingsRequestSortOrder>,

    /// A filter to return only the publisher that matches the given publisher name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_name: Option<String>,

    /// A filter to return only publishers that match the given publisher type exactly. Valid types are OCI, ORACLE, TRUSTED, STANDARD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_type: Option<String>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl ListAppCatalogListingsRequest {
    /// Create a new ListAppCatalogListingsRequest
    pub fn new() -> Self {
        Self {
            limit: None,

            page: None,

            sort_order: None,

            publisher_name: None,

            publisher_type: None,

            display_name: None,
        }
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

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListAppCatalogListingsRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set publisher_name
    pub fn set_publisher_name(mut self, value: Option<String>) -> Self {
        self.publisher_name = value;
        self
    }

    /// Set publisher_type
    pub fn set_publisher_type(mut self, value: Option<String>) -> Self {
        self.publisher_type = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
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

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListAppCatalogListingsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set publisher_name (unwraps Option)
    pub fn with_publisher_name(mut self, value: impl Into<String>) -> Self {
        self.publisher_name = Some(value.into());
        self
    }

    /// Set publisher_type (unwraps Option)
    pub fn with_publisher_type(mut self, value: impl Into<String>) -> Self {
        self.publisher_type = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}

impl Default for ListAppCatalogListingsRequest {
    fn default() -> Self {
        Self::new()
    }
}

