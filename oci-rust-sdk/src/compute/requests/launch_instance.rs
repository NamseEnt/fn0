use crate::compute::models::*;
use serde::{Deserialize, Serialize};

/// Required fields for LaunchInstanceRequest
pub struct LaunchInstanceRequestRequiredFields {
    /// Details for launching the instance
    pub launch_instance_details: LaunchInstanceDetails,
}

/// Request to launch a new compute instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceRequest {
    /// Details for launching the instance
    pub launch_instance_details: LaunchInstanceDetails,

    /// A token that uniquely identifies a request for idempotency
    pub opc_retry_token: Option<String>,
}

impl LaunchInstanceRequest {
    /// Create a new builder for LaunchInstanceRequest
    pub fn builder(required: LaunchInstanceRequestRequiredFields) -> LaunchInstanceRequestBuilder {
        LaunchInstanceRequestBuilder {
            launch_instance_details: required.launch_instance_details,
            opc_retry_token: None,
        }
    }

    /// Create a request from launch instance details
    pub fn new(launch_instance_details: LaunchInstanceDetails) -> Self {
        Self {
            launch_instance_details,
            opc_retry_token: None,
        }
    }
}

/// Builder for LaunchInstanceRequest
pub struct LaunchInstanceRequestBuilder {
    launch_instance_details: LaunchInstanceDetails,
    opc_retry_token: Option<String>,
}

impl LaunchInstanceRequestBuilder {
    /// Set the retry token for idempotency
    pub fn opc_retry_token(mut self, token: impl Into<String>) -> Self {
        self.opc_retry_token = Some(token.into());
        self
    }

    pub fn set_opc_retry_token(mut self, token: Option<impl Into<String>>) -> Self {
        self.opc_retry_token = token.map(|t| t.into());
        self
    }

    /// Build the request
    pub fn build(self) -> LaunchInstanceRequest {
        LaunchInstanceRequest {
            launch_instance_details: self.launch_instance_details,
            opc_retry_token: self.opc_retry_token,
        }
    }
}


/// Response from launching an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceResponse {
    /// The launched instance
    pub instance: Instance,

    /// Unique Oracle-assigned identifier for the request
    pub opc_request_id: Option<String>,

    /// Entity tag for the resource
    pub etag: Option<String>,
}
