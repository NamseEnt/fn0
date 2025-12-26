use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDrgAttachmentsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_id: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource (virtual circuit, VCN, IPSec tunnel, or remote peering connection) attached to the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// The type for the network resource attached to the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_type: Option<ListDrgAttachmentsRequestAttachmentType>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table assigned to the DRG attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_route_table_id: Option<String>,

    /// A filter to return only resources that match the given display name exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The field to sort by. You can provide one sort order ({@code sortOrder}). Default order for TIMECREATED is descending. Default order for DISPLAYNAME is ascending. The DISPLAYNAME sort order is case sensitive. <p> *Note:** In general, some \"List\" operations (for example, {@code ListInstances}) let you optionally filter by availability domain if the scope of the resource type is within a single availability domain. If you call one of these \"List\" operations without specifying an availability domain, the resources are grouped by availability domain, then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListDrgAttachmentsRequestSortBy>,

    /// The sort order to use, either ascending ({@code ASC}) or descending ({@code DESC}). The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListDrgAttachmentsRequestSortOrder>,

    /// A filter to return only resources that match the specified lifecycle state. The value is case insensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
}


/// Required fields for ListDrgAttachmentsRequest
pub struct ListDrgAttachmentsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListDrgAttachmentsRequest {
    /// Create a new ListDrgAttachmentsRequest with required fields
    pub fn new(required: ListDrgAttachmentsRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            vcn_id: None,

            drg_id: None,

            limit: None,

            page: None,

            network_id: None,

            attachment_type: None,

            drg_route_table_id: None,

            display_name: None,

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

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: Option<String>) -> Self {
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

    /// Set network_id
    pub fn set_network_id(mut self, value: Option<String>) -> Self {
        self.network_id = value;
        self
    }

    /// Set attachment_type
    pub fn set_attachment_type(mut self, value: Option<ListDrgAttachmentsRequestAttachmentType>) -> Self {
        self.attachment_type = value;
        self
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: Option<String>) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListDrgAttachmentsRequestSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListDrgAttachmentsRequestSortOrder>) -> Self {
        self.sort_order = value;
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

    /// Set drg_id (unwraps Option)
    pub fn with_drg_id(mut self, value: impl Into<String>) -> Self {
        self.drg_id = Some(value.into());
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

    /// Set network_id (unwraps Option)
    pub fn with_network_id(mut self, value: impl Into<String>) -> Self {
        self.network_id = Some(value.into());
        self
    }

    /// Set attachment_type (unwraps Option)
    pub fn with_attachment_type(mut self, value: ListDrgAttachmentsRequestAttachmentType) -> Self {
        self.attachment_type = Some(value);
        self
    }

    /// Set drg_route_table_id (unwraps Option)
    pub fn with_drg_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.drg_route_table_id = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListDrgAttachmentsRequestSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListDrgAttachmentsRequestSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }
}


