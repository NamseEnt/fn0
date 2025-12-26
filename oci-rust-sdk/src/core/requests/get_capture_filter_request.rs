use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptureFilterRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the capture filter.
    pub capture_filter_id: String,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for GetCaptureFilterRequest
pub struct GetCaptureFilterRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the capture filter.
    pub capture_filter_id: String,
}

impl GetCaptureFilterRequest {
    /// Create a new GetCaptureFilterRequest with required fields
    pub fn new(required: GetCaptureFilterRequestRequired) -> Self {
        Self {
            capture_filter_id: required.capture_filter_id,

            opc_request_id: None,
}
    }

    /// Set capture_filter_id
    pub fn set_capture_filter_id(mut self, value: String) -> Self {
        self.capture_filter_id = value;
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


