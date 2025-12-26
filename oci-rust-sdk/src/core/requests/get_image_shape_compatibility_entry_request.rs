use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImageShapeCompatibilityEntryRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetImageShapeCompatibilityEntryRequest
pub struct GetImageShapeCompatibilityEntryRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,
}

impl GetImageShapeCompatibilityEntryRequest {
    /// Create a new GetImageShapeCompatibilityEntryRequest with required fields
    pub fn new(required: GetImageShapeCompatibilityEntryRequestRequired) -> Self {
        Self {
            image_id: required.image_id,

            shape_name: required.shape_name,

            opc_request_id: None,
}
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
        self
    }

    /// Set shape_name
    pub fn set_shape_name(mut self, value: String) -> Self {
        self.shape_name = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


