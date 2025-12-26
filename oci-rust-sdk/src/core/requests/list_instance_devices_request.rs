use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInstanceDevicesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,

    /// A filter to return only available devices or only used devices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_available: Option<bool>,

    /// A filter to return only devices that match the given name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

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
    pub sort_by: Option<ListInstanceDevicesRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListInstanceDevicesRequestSortOrder>,
}


/// Required fields for ListInstanceDevicesRequest
pub struct ListInstanceDevicesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl ListInstanceDevicesRequest {
    /// Create a new ListInstanceDevicesRequest with required fields
    pub fn new(required: ListInstanceDevicesRequestRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            is_available: None,

            name: None,

            limit: None,

            page: None,

            opc_request_id: None,

            sort_by: None,

            sort_order: None,
}
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set is_available
    pub fn set_is_available(mut self, value: Option<bool>) -> Self {
        self.is_available = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: Option<String>) -> Self {
        self.name = value;
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
    pub fn set_sort_by(mut self, value: Option<ListInstanceDevicesRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListInstanceDevicesRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set is_available (unwraps Option)
    pub fn with_is_available(mut self, value: bool) -> Self {
        self.is_available = Some(value);
        self
    }

    /// Set name (unwraps Option)
    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
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
    pub fn with_sort_by(mut self, value: ListInstanceDevicesRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListInstanceDevicesRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }
}


