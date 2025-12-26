use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachServiceIdResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ServiceGateway instance.
    pub service_gateway: ServiceGateway,
}


/// Required fields for AttachServiceIdResponse
pub struct AttachServiceIdResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ServiceGateway instance.
    pub service_gateway: ServiceGateway,
}

impl AttachServiceIdResponse {
    /// Create a new AttachServiceIdResponse with required fields
    pub fn new(required: AttachServiceIdResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            service_gateway: required.service_gateway,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set service_gateway
    pub fn set_service_gateway(mut self, value: ServiceGateway) -> Self {
        self.service_gateway = value;
        self
    }
}


