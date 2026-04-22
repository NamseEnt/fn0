use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Security context for container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecurityContextDetails {
    pub security_context_type: String,
}

/// Required fields for CreateSecurityContextDetails
pub struct CreateSecurityContextDetailsRequired {
    pub security_context_type: String,
}

impl CreateSecurityContextDetails {
    /// Create a new CreateSecurityContextDetails with required fields
    pub fn new(required: CreateSecurityContextDetailsRequired) -> Self {
        Self {
            security_context_type: required.security_context_type,
        }
    }

    /// Set security_context_type
    pub fn set_security_context_type(mut self, value: String) -> Self {
        self.security_context_type = value;
        self
    }
}
