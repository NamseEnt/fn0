use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVtapsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VTAP source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VTAP target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,

    /// The IP address of the VTAP target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,

    /// Indicates whether to list all VTAPs or only running VTAPs. <p> When {@code FALSE}, lists ALL running and stopped VTAPs. * When {@code TRUE}, lists only running VTAPs (VTAPs where isVtapEnabled = {@code TRUE}).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_vtap_enabled: Option<bool>,

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
    pub sort_by: Option<ListVtapsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListVtapsRequestSortOrder>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A filter to return only resources that match the given VTAP administrative lifecycle state. The state value is case-insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
}


/// Required fields for ListVtapsRequest
pub struct ListVtapsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListVtapsRequest {
    /// Create a new ListVtapsRequest with required fields
    pub fn new(required: ListVtapsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            vcn_id: None,

            source: None,

            target_id: None,

            target_ip: None,

            is_vtap_enabled: None,

            limit: None,

            page: None,

            opc_request_id: None,

            sort_by: None,

            sort_order: None,

            display_name: None,

            lifecycle_state: None,
}
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set source
    pub fn set_source(mut self, value: Option<String>) -> Self {
        self.source = value;
        self
    }

    /// Set target_id
    pub fn set_target_id(mut self, value: Option<String>) -> Self {
        self.target_id = value;
        self
    }

    /// Set target_ip
    pub fn set_target_ip(mut self, value: Option<String>) -> Self {
        self.target_ip = value;
        self
    }

    /// Set is_vtap_enabled
    pub fn set_is_vtap_enabled(mut self, value: Option<bool>) -> Self {
        self.is_vtap_enabled = value;
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
    pub fn set_sort_by(mut self, value: Option<ListVtapsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListVtapsRequestSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set vcn_id (unwraps Option)
    pub fn with_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.vcn_id = Some(value.into());
        self
    }

    /// Set source (unwraps Option)
    pub fn with_source(mut self, value: impl Into<String>) -> Self {
        self.source = Some(value.into());
        self
    }

    /// Set target_id (unwraps Option)
    pub fn with_target_id(mut self, value: impl Into<String>) -> Self {
        self.target_id = Some(value.into());
        self
    }

    /// Set target_ip (unwraps Option)
    pub fn with_target_ip(mut self, value: impl Into<String>) -> Self {
        self.target_ip = Some(value.into());
        self
    }

    /// Set is_vtap_enabled (unwraps Option)
    pub fn with_is_vtap_enabled(mut self, value: bool) -> Self {
        self.is_vtap_enabled = Some(value);
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
    pub fn with_sort_by(mut self, value: ListVtapsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListVtapsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }
}


