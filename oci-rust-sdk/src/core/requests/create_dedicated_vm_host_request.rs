use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDedicatedVmHostRequest {
    /// The details for creating a new dedicated virtual machine host.
    pub create_dedicated_vm_host_details: CreateDedicatedVmHostDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}


/// Required fields for CreateDedicatedVmHostRequest
pub struct CreateDedicatedVmHostRequestRequired {
    /// The details for creating a new dedicated virtual machine host.
    pub create_dedicated_vm_host_details: CreateDedicatedVmHostDetails,
}

impl CreateDedicatedVmHostRequest {
    /// Create a new CreateDedicatedVmHostRequest with required fields
    pub fn new(required: CreateDedicatedVmHostRequestRequired) -> Self {
        Self {
            create_dedicated_vm_host_details: required.create_dedicated_vm_host_details,

            opc_request_id: None,

            opc_retry_token: None,
}
    }

    /// Set create_dedicated_vm_host_details
    pub fn set_create_dedicated_vm_host_details(mut self, value: CreateDedicatedVmHostDetails) -> Self {
        self.create_dedicated_vm_host_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}


