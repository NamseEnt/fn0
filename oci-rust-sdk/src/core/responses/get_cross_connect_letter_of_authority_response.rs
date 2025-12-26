use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectLetterOfAuthorityResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.LetterOfAuthority instance.
    pub letter_of_authority: LetterOfAuthority,
}

/// Required fields for GetCrossConnectLetterOfAuthorityResponse
pub struct GetCrossConnectLetterOfAuthorityResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.LetterOfAuthority instance.
    pub letter_of_authority: LetterOfAuthority,
}

impl GetCrossConnectLetterOfAuthorityResponse {
    /// Create a new GetCrossConnectLetterOfAuthorityResponse with required fields
    pub fn new(required: GetCrossConnectLetterOfAuthorityResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            letter_of_authority: required.letter_of_authority,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set letter_of_authority
    pub fn set_letter_of_authority(mut self, value: LetterOfAuthority) -> Self {
        self.letter_of_authority = value;
        self
    }
}
