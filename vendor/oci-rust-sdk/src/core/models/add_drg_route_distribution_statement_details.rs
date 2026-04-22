use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used to add a route distribution statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteDistributionStatementDetails {
    /// The action is applied only if all of the match criteria is met.
    pub match_criteria: Vec<DrgRouteDistributionMatchCriteria>,

    /// Accept: import/export the route \"as is\"
    pub action: AddDrgRouteDistributionStatementDetailsAction,

    /// This field is used to specify the priority of each statement in a route distribution. The priority will be represented as a number between 0 and 65535 where a lower number indicates a higher priority. When a route is processed, statements are applied in the order defined by their priority. The first matching rule dictates the action that will be taken on the route. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub priority: i64,
}

/// Required fields for AddDrgRouteDistributionStatementDetails
pub struct AddDrgRouteDistributionStatementDetailsRequired {
    /// The action is applied only if all of the match criteria is met.
    pub match_criteria: Vec<DrgRouteDistributionMatchCriteria>,

    /// Accept: import/export the route \"as is\"
    pub action: AddDrgRouteDistributionStatementDetailsAction,

    /// This field is used to specify the priority of each statement in a route distribution. The priority will be represented as a number between 0 and 65535 where a lower number indicates a higher priority. When a route is processed, statements are applied in the order defined by their priority. The first matching rule dictates the action that will be taken on the route. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub priority: i64,
}

impl AddDrgRouteDistributionStatementDetails {
    /// Create a new AddDrgRouteDistributionStatementDetails with required fields
    pub fn new(required: AddDrgRouteDistributionStatementDetailsRequired) -> Self {
        Self {
            match_criteria: required.match_criteria,

            action: required.action,

            priority: required.priority,
        }
    }

    /// Set match_criteria
    pub fn set_match_criteria(mut self, value: Vec<DrgRouteDistributionMatchCriteria>) -> Self {
        self.match_criteria = value;
        self
    }

    /// Set action
    pub fn set_action(mut self, value: AddDrgRouteDistributionStatementDetailsAction) -> Self {
        self.action = value;
        self
    }

    /// Set priority
    pub fn set_priority(mut self, value: i64) -> Self {
        self.priority = value;
        self
    }
}
