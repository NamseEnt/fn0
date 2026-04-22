use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used to update route rules in a DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteRulesDetails {
    /// The DRG rute rules to update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rules: Option<Vec<UpdateDrgRouteRuleDetails>>,
}

impl UpdateDrgRouteRulesDetails {
    /// Create a new UpdateDrgRouteRulesDetails
    pub fn new() -> Self {
        Self { route_rules: None }
    }

    /// Set route_rules
    pub fn set_route_rules(mut self, value: Option<Vec<UpdateDrgRouteRuleDetails>>) -> Self {
        self.route_rules = value;
        self
    }

    /// Set route_rules (unwraps Option)
    pub fn with_route_rules(mut self, value: Vec<UpdateDrgRouteRuleDetails>) -> Self {
        self.route_rules = Some(value);
        self
    }
}

impl Default for UpdateDrgRouteRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
