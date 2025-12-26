use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details request to add statements to a route distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteDistributionStatementsDetails {
    /// The collection of route distribution statements to insert into the route distribution.
    pub statements: Vec<AddDrgRouteDistributionStatementDetails>,
}

/// Required fields for AddDrgRouteDistributionStatementsDetails
pub struct AddDrgRouteDistributionStatementsDetailsRequired {
    /// The collection of route distribution statements to insert into the route distribution.
    pub statements: Vec<AddDrgRouteDistributionStatementDetails>,
}

impl AddDrgRouteDistributionStatementsDetails {
    /// Create a new AddDrgRouteDistributionStatementsDetails with required fields
    pub fn new(required: AddDrgRouteDistributionStatementsDetailsRequired) -> Self {
        Self {
            statements: required.statements,
        }
    }

    /// Set statements
    pub fn set_statements(mut self, value: Vec<AddDrgRouteDistributionStatementDetails>) -> Self {
        self.statements = value;
        self
    }
}
