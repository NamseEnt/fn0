use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used in a request to add static routes to a DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteRulesDetails {
    /// The collection of static rules used to insert routes into the DRG route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rules: Option<Vec<AddDrgRouteRuleDetails>>,
}

impl AddDrgRouteRulesDetails {
    /// Create a new AddDrgRouteRulesDetails
    pub fn new() -> Self {
        Self { route_rules: None }
    }

    /// Set route_rules
    pub fn set_route_rules(mut self, value: Option<Vec<AddDrgRouteRuleDetails>>) -> Self {
        self.route_rules = value;
        self
    }

    /// Set route_rules (unwraps Option)
    pub fn with_route_rules(mut self, value: Vec<AddDrgRouteRuleDetails>) -> Self {
        self.route_rules = Some(value);
        self
    }
}

impl Default for AddDrgRouteRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
