use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDedicatedVmHostResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DedicatedVmHost instance.
    pub dedicated_vm_host: DedicatedVmHost,
}


/// Required fields for GetDedicatedVmHostResponse
pub struct GetDedicatedVmHostResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DedicatedVmHost instance.
    pub dedicated_vm_host: DedicatedVmHost,
}

impl GetDedicatedVmHostResponse {
    /// Create a new GetDedicatedVmHostResponse with required fields
    pub fn new(required: GetDedicatedVmHostResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            dedicated_vm_host: required.dedicated_vm_host,
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

    /// Set dedicated_vm_host
    pub fn set_dedicated_vm_host(mut self, value: DedicatedVmHost) -> Self {
        self.dedicated_vm_host = value;
        self
    }
}


