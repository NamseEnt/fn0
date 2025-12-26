use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllDrgAttachmentsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The type for the network resource attached to the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_type: Option<GetAllDrgAttachmentsRequestAttachmentType>,

    /// Whether the DRG attachment lives in a different tenancy than the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cross_tenancy: Option<bool>,
}

/// Required fields for GetAllDrgAttachmentsRequest
pub struct GetAllDrgAttachmentsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,
}

impl GetAllDrgAttachmentsRequest {
    /// Create a new GetAllDrgAttachmentsRequest with required fields
    pub fn new(required: GetAllDrgAttachmentsRequestRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            opc_request_id: None,

            limit: None,

            page: None,

            attachment_type: None,

            is_cross_tenancy: None,
        }
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
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

    /// Set attachment_type
    pub fn set_attachment_type(
        mut self,
        value: Option<GetAllDrgAttachmentsRequestAttachmentType>,
    ) -> Self {
        self.attachment_type = value;
        self
    }

    /// Set is_cross_tenancy
    pub fn set_is_cross_tenancy(mut self, value: Option<bool>) -> Self {
        self.is_cross_tenancy = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
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

    /// Set attachment_type (unwraps Option)
    pub fn with_attachment_type(
        mut self,
        value: GetAllDrgAttachmentsRequestAttachmentType,
    ) -> Self {
        self.attachment_type = Some(value);
        self
    }

    /// Set is_cross_tenancy (unwraps Option)
    pub fn with_is_cross_tenancy(mut self, value: bool) -> Self {
        self.is_cross_tenancy = Some(value);
        self
    }
}
