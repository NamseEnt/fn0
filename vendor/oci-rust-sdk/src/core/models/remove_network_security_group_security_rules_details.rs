use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNetworkSecurityGroupSecurityRulesDetails {
    /// The Oracle-assigned ID of each {@link SecurityRule} to be deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_rule_ids: Option<Vec<String>>,
}

impl RemoveNetworkSecurityGroupSecurityRulesDetails {
    /// Create a new RemoveNetworkSecurityGroupSecurityRulesDetails
    pub fn new() -> Self {
        Self {
            security_rule_ids: None,
        }
    }

    /// Set security_rule_ids
    pub fn set_security_rule_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.security_rule_ids = value;
        self
    }

    /// Set security_rule_ids (unwraps Option)
    pub fn with_security_rule_ids(mut self, value: Vec<String>) -> Self {
        self.security_rule_ids = Some(value);
        self
    }
}

impl Default for RemoveNetworkSecurityGroupSecurityRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
