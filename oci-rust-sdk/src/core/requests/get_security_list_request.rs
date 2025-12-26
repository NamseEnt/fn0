use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSecurityListRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the security list.
    pub security_list_id: String,
}

/// Required fields for GetSecurityListRequest
pub struct GetSecurityListRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the security list.
    pub security_list_id: String,
}

impl GetSecurityListRequest {
    /// Create a new GetSecurityListRequest with required fields
    pub fn new(required: GetSecurityListRequestRequired) -> Self {
        Self {
            security_list_id: required.security_list_id,
        }
    }

    /// Set security_list_id
    pub fn set_security_list_id(mut self, value: String) -> Self {
        self.security_list_id = value;
        self
    }
}
