use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A single statement within a route distribution. All match criteria in a statement must be met for the action to take place.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRouteDistributionStatement {
    /// The action is applied only if all of the match criteria is met. If there are no match criteria in a statement, any input is considered a match and the action is applied.
    pub match_criteria: Vec<DrgRouteDistributionMatchCriteria>,

    /// {@code ACCEPT} indicates the route should be imported or exported as-is.
    pub action: DrgRouteDistributionStatementAction,

    /// This field specifies the priority of each statement in a route distribution. Priorities must be unique within a particular route distribution. The priority will be represented as a number between 0 and 65535 where a lower number indicates a higher priority. When a route is processed, statements are applied in the order defined by their priority. The first matching rule dictates the action that will be taken on the route. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub priority: i64,

    /// The Oracle-assigned ID of the route distribution statement.
    pub id: String,
}

/// Required fields for DrgRouteDistributionStatement
pub struct DrgRouteDistributionStatementRequired {
    /// The action is applied only if all of the match criteria is met. If there are no match criteria in a statement, any input is considered a match and the action is applied.
    pub match_criteria: Vec<DrgRouteDistributionMatchCriteria>,

    /// {@code ACCEPT} indicates the route should be imported or exported as-is.
    pub action: DrgRouteDistributionStatementAction,

    /// This field specifies the priority of each statement in a route distribution. Priorities must be unique within a particular route distribution. The priority will be represented as a number between 0 and 65535 where a lower number indicates a higher priority. When a route is processed, statements are applied in the order defined by their priority. The first matching rule dictates the action that will be taken on the route. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub priority: i64,

    /// The Oracle-assigned ID of the route distribution statement.
    pub id: String,
}

impl DrgRouteDistributionStatement {
    /// Create a new DrgRouteDistributionStatement with required fields
    pub fn new(required: DrgRouteDistributionStatementRequired) -> Self {
        Self {
            match_criteria: required.match_criteria,

            action: required.action,

            priority: required.priority,

            id: required.id,
        }
    }

    /// Set match_criteria
    pub fn set_match_criteria(mut self, value: Vec<DrgRouteDistributionMatchCriteria>) -> Self {
        self.match_criteria = value;
        self
    }

    /// Set action
    pub fn set_action(mut self, value: DrgRouteDistributionStatementAction) -> Self {
        self.action = value;
        self
    }

    /// Set priority
    pub fn set_priority(mut self, value: i64) -> Self {
        self.priority = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }
}
