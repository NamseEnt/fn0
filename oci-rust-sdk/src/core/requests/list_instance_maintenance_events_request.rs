use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInstanceMaintenanceEventsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The OCID of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// A filter to only return resources that match the given lifecycle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// A filter to only return resources that have a matching correlationToken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_token: Option<String>,

    /// A filter to only return resources that match the given instance action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_action: Option<String>,

    /// Starting range to return the maintenances which are not completed (date-time is in [RFC3339](https://tools.ietf.org/html/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_window_start_greater_than_or_equal_to: Option<DateTime<Utc>>,

    /// Ending range to return the maintenances which are not completed (date-time is in [RFC3339](https://tools.ietf.org/html/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_window_start_less_than_or_equal_to: Option<DateTime<Utc>>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListInstanceMaintenanceEventsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListInstanceMaintenanceEventsRequestSortOrder>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for ListInstanceMaintenanceEventsRequest
pub struct ListInstanceMaintenanceEventsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListInstanceMaintenanceEventsRequest {
    /// Create a new ListInstanceMaintenanceEventsRequest with required fields
    pub fn new(required: ListInstanceMaintenanceEventsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            instance_id: None,

            lifecycle_state: None,

            correlation_token: None,

            instance_action: None,

            time_window_start_greater_than_or_equal_to: None,

            time_window_start_less_than_or_equal_to: None,

            limit: None,

            page: None,

            sort_by: None,

            sort_order: None,

            opc_request_id: None,
}
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set correlation_token
    pub fn set_correlation_token(mut self, value: Option<String>) -> Self {
        self.correlation_token = value;
        self
    }

    /// Set instance_action
    pub fn set_instance_action(mut self, value: Option<String>) -> Self {
        self.instance_action = value;
        self
    }

    /// Set time_window_start_greater_than_or_equal_to
    pub fn set_time_window_start_greater_than_or_equal_to(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_window_start_greater_than_or_equal_to = value;
        self
    }

    /// Set time_window_start_less_than_or_equal_to
    pub fn set_time_window_start_less_than_or_equal_to(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_window_start_less_than_or_equal_to = value;
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
    pub fn set_sort_by(mut self, value: Option<ListInstanceMaintenanceEventsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListInstanceMaintenanceEventsRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }

    /// Set correlation_token (unwraps Option)
    pub fn with_correlation_token(mut self, value: impl Into<String>) -> Self {
        self.correlation_token = Some(value.into());
        self
    }

    /// Set instance_action (unwraps Option)
    pub fn with_instance_action(mut self, value: impl Into<String>) -> Self {
        self.instance_action = Some(value.into());
        self
    }

    /// Set time_window_start_greater_than_or_equal_to (unwraps Option)
    pub fn with_time_window_start_greater_than_or_equal_to(mut self, value: DateTime<Utc>) -> Self {
        self.time_window_start_greater_than_or_equal_to = Some(value);
        self
    }

    /// Set time_window_start_less_than_or_equal_to (unwraps Option)
    pub fn with_time_window_start_less_than_or_equal_to(mut self, value: DateTime<Utc>) -> Self {
        self.time_window_start_less_than_or_equal_to = Some(value);
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
    pub fn with_sort_by(mut self, value: ListInstanceMaintenanceEventsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListInstanceMaintenanceEventsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


