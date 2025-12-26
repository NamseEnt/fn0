use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContainerInstanceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance.
    pub container_instance_id: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for GetContainerInstanceRequest
pub struct GetContainerInstanceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance.
    pub container_instance_id: String,
}

impl GetContainerInstanceRequest {
    /// Create a new GetContainerInstanceRequest with required fields
    pub fn new(required: GetContainerInstanceRequestRequired) -> Self {
        Self {
            container_instance_id: required.container_instance_id,

            opc_request_id: None,
        }
    }

    /// Set container_instance_id
    pub fn set_container_instance_id(mut self, value: String) -> Self {
        self.container_instance_id = value;
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
    /// Convert this request's query parameters to a vector of key-value pairs.
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}
