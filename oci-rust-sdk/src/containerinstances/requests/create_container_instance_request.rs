use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerInstanceRequest {
    /// Details for the new container instance.
    pub create_container_instance_details: CreateContainerInstanceDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations. For example, if a resource has been deleted and purged from the system, then a retry of the original creation request might be rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for CreateContainerInstanceRequest
pub struct CreateContainerInstanceRequestRequired {
    /// Details for the new container instance.
    pub create_container_instance_details: CreateContainerInstanceDetails,
}

impl CreateContainerInstanceRequest {
    /// Create a new CreateContainerInstanceRequest with required fields
    pub fn new(required: CreateContainerInstanceRequestRequired) -> Self {
        Self {
            create_container_instance_details: required.create_container_instance_details,

            opc_retry_token: None,

            opc_request_id: None,
}
    }

    /// Set create_container_instance_details
    pub fn set_create_container_instance_details(mut self, value: CreateContainerInstanceDetails) -> Self {
        self.create_container_instance_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
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


