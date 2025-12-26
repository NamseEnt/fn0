use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used in a request to remove static routes from a DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDrgRouteRulesDetails {
    /// The Oracle-assigned ID of each DRG route rule to be deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rule_ids: Option<Vec<String>>,
}

impl RemoveDrgRouteRulesDetails {
    /// Create a new RemoveDrgRouteRulesDetails
    pub fn new() -> Self {
        Self {
            route_rule_ids: None,
        }
    }

    /// Set route_rule_ids
    pub fn set_route_rule_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.route_rule_ids = value;
        self
    }

    /// Set route_rule_ids (unwraps Option)
    pub fn with_route_rule_ids(mut self, value: Vec<String>) -> Self {
        self.route_rule_ids = Some(value);
        self
    }
}

impl Default for RemoveDrgRouteRulesDetails {
    fn default() -> Self {
        Self::new()
    }
}
