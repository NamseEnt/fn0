use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainersRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment in which to list resources.
    pub compartment_id: String,

    /// A filter to only return resources that match the given lifecycle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// A filter to return only resources that match the entire display name given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_instance_id: Option<String>,

    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the opc-next-page response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// The sort order to use (ASC) or (DESC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,

    /// The field to sort by. You can provide one sort order. Default order for timeCreated is descending. Default order for displayName is ascending. If you don't specify a value, timeCreated is the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListContainersRequestSortBy>,
}

/// Required fields for ListContainersRequest
pub struct ListContainersRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment in which to list resources.
    pub compartment_id: String,
}

impl ListContainersRequest {
    /// Create a new ListContainersRequest with required fields
    pub fn new(required: ListContainersRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            lifecycle_state: None,

            display_name: None,

            container_instance_id: None,

            availability_domain: None,

            limit: None,

            page: None,

            opc_request_id: None,

            sort_order: None,

            sort_by: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set container_instance_id
    pub fn set_container_instance_id(mut self, value: Option<String>) -> Self {
        self.container_instance_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
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

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<SortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListContainersRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set container_instance_id (unwraps Option)
    pub fn with_container_instance_id(mut self, value: impl Into<String>) -> Self {
        self.container_instance_id = Some(value.into());
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
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

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: SortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListContainersRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }
    /// Convert this request's query parameters to a vector of key-value pairs.
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
