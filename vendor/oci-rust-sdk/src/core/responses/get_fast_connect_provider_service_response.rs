use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFastConnectProviderServiceResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.FastConnectProviderService instance.
    pub fast_connect_provider_service: FastConnectProviderService,
}

/// Required fields for GetFastConnectProviderServiceResponse
pub struct GetFastConnectProviderServiceResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.FastConnectProviderService instance.
    pub fast_connect_provider_service: FastConnectProviderService,
}

impl GetFastConnectProviderServiceResponse {
    /// Create a new GetFastConnectProviderServiceResponse with required fields
    pub fn new(required: GetFastConnectProviderServiceResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            fast_connect_provider_service: required.fast_connect_provider_service,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set fast_connect_provider_service
    pub fn set_fast_connect_provider_service(mut self, value: FastConnectProviderService) -> Self {
        self.fast_connect_provider_service = value;
        self
    }
}
