use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceConfigurationResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceConfiguration instance.
    pub instance_configuration: InstanceConfiguration,
}

/// Required fields for CreateInstanceConfigurationResponse
pub struct CreateInstanceConfigurationResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstanceConfiguration instance.
    pub instance_configuration: InstanceConfiguration,
}

impl CreateInstanceConfigurationResponse {
    /// Create a new CreateInstanceConfigurationResponse with required fields
    pub fn new(required: CreateInstanceConfigurationResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            instance_configuration: required.instance_configuration,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set instance_configuration
    pub fn set_instance_configuration(mut self, value: InstanceConfiguration) -> Self {
        self.instance_configuration = value;
        self
    }
}
