use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNetworkSecurityGroupSecurityRulesDetails {
    /// An array of security rules to add to the NSG. You can add up to 25 rules in a single {@code AddNetworkSecurityGroupSecurityRules} operation. Adding more than 25 rules requires multiple operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_rules: Option<Vec<AddSecurityRuleDetails>>,
}

impl AddNetworkSecurityGroupSecurityRulesDetails {
    /// Create a new AddNetworkSecurityGroupSecurityRulesDetails
    pub fn new() -> Self {
        Self {
            security_rules: None,
        }
    }

    /// Set security_rules
    pub fn set_security_rules(mut self, value: Option<Vec<AddSecurityRuleDetails>>) -> Self {
        self.security_rules = value;
        self
    }

    /// Set security_rules (unwraps Option)
    pub fn with_security_rules(mut self, value: Vec<AddSecurityRuleDetails>) -> Self {
        self.security_rules = Some(value);
        self
    }
}

impl Default for AddNetworkSecurityGroupSecurityRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
