use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstancePoolRequest {
    /// Instance pool creation details
    pub create_instance_pool_details: CreateInstancePoolDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for CreateInstancePoolRequest
pub struct CreateInstancePoolRequestRequired {
    /// Instance pool creation details
    pub create_instance_pool_details: CreateInstancePoolDetails,
}

impl CreateInstancePoolRequest {
    /// Create a new CreateInstancePoolRequest with required fields
    pub fn new(required: CreateInstancePoolRequestRequired) -> Self {
        Self {
            create_instance_pool_details: required.create_instance_pool_details,

            opc_retry_token: None,
        }
    }

    /// Set create_instance_pool_details
    pub fn set_create_instance_pool_details(mut self, value: CreateInstancePoolDetails) -> Self {
        self.create_instance_pool_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}
