use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkSecurityGroupSecurityRulesDetails {
    /// The NSG security rules to update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_rules: Option<Vec<UpdateSecurityRuleDetails>>,
}

impl UpdateNetworkSecurityGroupSecurityRulesDetails {
    /// Create a new UpdateNetworkSecurityGroupSecurityRulesDetails
    pub fn new() -> Self {
        Self {
            security_rules: None,
        }
    }

    /// Set security_rules
    pub fn set_security_rules(mut self, value: Option<Vec<UpdateSecurityRuleDetails>>) -> Self {
        self.security_rules = value;
        self
    }

    /// Set security_rules (unwraps Option)
    pub fn with_security_rules(mut self, value: Vec<UpdateSecurityRuleDetails>) -> Self {
        self.security_rules = Some(value);
        self
    }
}

impl Default for UpdateNetworkSecurityGroupSecurityRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
