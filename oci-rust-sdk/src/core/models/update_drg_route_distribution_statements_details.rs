use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details request to update statements in a route distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteDistributionStatementsDetails {
    /// The route distribution statements to update, and the details to be updated.
    pub statements: Vec<UpdateDrgRouteDistributionStatementDetails>,
}

/// Required fields for UpdateDrgRouteDistributionStatementsDetails
pub struct UpdateDrgRouteDistributionStatementsDetailsRequired {
    /// The route distribution statements to update, and the details to be updated.
    pub statements: Vec<UpdateDrgRouteDistributionStatementDetails>,
}

impl UpdateDrgRouteDistributionStatementsDetails {
    /// Create a new UpdateDrgRouteDistributionStatementsDetails with required fields
    pub fn new(required: UpdateDrgRouteDistributionStatementsDetailsRequired) -> Self {
        Self {
            statements: required.statements,
        }
    }

    /// Set statements
    pub fn set_statements(
        mut self,
        value: Vec<UpdateDrgRouteDistributionStatementDetails>,
    ) -> Self {
        self.statements = value;
        self
    }
}
