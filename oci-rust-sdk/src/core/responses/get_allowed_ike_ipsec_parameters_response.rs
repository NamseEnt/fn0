use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllowedIkeIPSecParametersResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AllowedIkeIPSecParameters instance.
    pub allowed_ike_ipsec_parameters: AllowedIkeIPSecParameters,
}


/// Required fields for GetAllowedIkeIPSecParametersResponse
pub struct GetAllowedIkeIPSecParametersResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AllowedIkeIPSecParameters instance.
    pub allowed_ike_ipsec_parameters: AllowedIkeIPSecParameters,
}

impl GetAllowedIkeIPSecParametersResponse {
    /// Create a new GetAllowedIkeIPSecParametersResponse with required fields
    pub fn new(required: GetAllowedIkeIPSecParametersResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            allowed_ike_ipsec_parameters: required.allowed_ike_ipsec_parameters,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set allowed_ike_ipsec_parameters
    pub fn set_allowed_ike_ipsec_parameters(mut self, value: AllowedIkeIPSecParameters) -> Self {
        self.allowed_ike_ipsec_parameters = value;
        self
    }
}


