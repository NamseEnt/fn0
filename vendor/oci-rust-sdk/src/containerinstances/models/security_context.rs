use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Security context for container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityContext {
    pub security_context_type: String,
}

/// Required fields for SecurityContext
pub struct SecurityContextRequired {
    pub security_context_type: String,
}

impl SecurityContext {
    /// Create a new SecurityContext with required fields
    pub fn new(required: SecurityContextRequired) -> Self {
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
