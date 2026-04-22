use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrieveLogsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub container_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// Returns the logs for the previous run of the container in a pod. If the container fails, this parameter can help you to determine the root cause of the failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_previous: Option<bool>,
}

/// Required fields for RetrieveLogsRequest
pub struct RetrieveLogsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub container_id: String,
}

impl RetrieveLogsRequest {
    /// Create a new RetrieveLogsRequest with required fields
    pub fn new(required: RetrieveLogsRequestRequired) -> Self {
        Self {
            container_id: required.container_id,

            opc_request_id: None,

            is_previous: None,
        }
    }

    /// Set container_id
    pub fn set_container_id(mut self, value: String) -> Self {
        self.container_id = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set is_previous
    pub fn set_is_previous(mut self, value: Option<bool>) -> Self {
        self.is_previous = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set is_previous (unwraps Option)
    pub fn with_is_previous(mut self, value: bool) -> Self {
        self.is_previous = Some(value);
        self
    }
    /// Convert this request's query parameters to a vector of key-value pairs.
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
