use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFastConnectProviderServiceKeyResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.FastConnectProviderServiceKey instance.
    pub fast_connect_provider_service_key: FastConnectProviderServiceKey,
}

/// Required fields for GetFastConnectProviderServiceKeyResponse
pub struct GetFastConnectProviderServiceKeyResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.FastConnectProviderServiceKey instance.
    pub fast_connect_provider_service_key: FastConnectProviderServiceKey,
}

impl GetFastConnectProviderServiceKeyResponse {
    /// Create a new GetFastConnectProviderServiceKeyResponse with required fields
    pub fn new(required: GetFastConnectProviderServiceKeyResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            fast_connect_provider_service_key: required.fast_connect_provider_service_key,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set fast_connect_provider_service_key
    pub fn set_fast_connect_provider_service_key(
        mut self,
        value: FastConnectProviderServiceKey,
    ) -> Self {
        self.fast_connect_provider_service_key = value;
        self
    }
}
