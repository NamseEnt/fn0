use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Route distribution statements to update in the route distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteDistributionStatementDetails {
    /// The Oracle-assigned ID of each route distribution statement to be updated.
    pub id: String,

    /// The action is applied only if all of the match criteria is met.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_criteria: Option<Vec<DrgRouteDistributionMatchCriteria>>,

    /// The priority of the statement you'd like to update. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
}

/// Required fields for UpdateDrgRouteDistributionStatementDetails
pub struct UpdateDrgRouteDistributionStatementDetailsRequired {
    /// The Oracle-assigned ID of each route distribution statement to be updated.
    pub id: String,
}

impl UpdateDrgRouteDistributionStatementDetails {
    /// Create a new UpdateDrgRouteDistributionStatementDetails with required fields
    pub fn new(required: UpdateDrgRouteDistributionStatementDetailsRequired) -> Self {
        Self {
            id: required.id,

            match_criteria: None,

            priority: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set match_criteria
    pub fn set_match_criteria(
        mut self,
        value: Option<Vec<DrgRouteDistributionMatchCriteria>>,
    ) -> Self {
        self.match_criteria = value;
        self
    }

    /// Set priority
    pub fn set_priority(mut self, value: Option<i64>) -> Self {
        self.priority = value;
        self
    }

    /// Set match_criteria (unwraps Option)
    pub fn with_match_criteria(mut self, value: Vec<DrgRouteDistributionMatchCriteria>) -> Self {
        self.match_criteria = Some(value);
        self
    }

    /// Set priority (unwraps Option)
    pub fn with_priority(mut self, value: i64) -> Self {
        self.priority = Some(value);
        self
    }
}
