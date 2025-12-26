use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWindowsInstanceInitialCredentialsResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceCredentials instance.
    pub instance_credentials: InstanceCredentials,
}


/// Required fields for GetWindowsInstanceInitialCredentialsResponse
pub struct GetWindowsInstanceInitialCredentialsResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceCredentials instance.
    pub instance_credentials: InstanceCredentials,
}

impl GetWindowsInstanceInitialCredentialsResponse {
    /// Create a new GetWindowsInstanceInitialCredentialsResponse with required fields
    pub fn new(required: GetWindowsInstanceInitialCredentialsResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            instance_credentials: required.instance_credentials,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_credentials
    pub fn set_instance_credentials(mut self, value: InstanceCredentials) -> Self {
        self.instance_credentials = value;
        self
    }
}


