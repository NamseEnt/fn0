use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details request to remove statements from a route distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDrgRouteDistributionStatementsDetails {
    /// The Oracle-assigned ID of each route distribution to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement_ids: Option<Vec<String>>,
}

impl RemoveDrgRouteDistributionStatementsDetails {
    /// Create a new RemoveDrgRouteDistributionStatementsDetails
    pub fn new() -> Self {
        Self {
            statement_ids: None,
        }
    }

    /// Set statement_ids
    pub fn set_statement_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.statement_ids = value;
        self
    }

    /// Set statement_ids (unwraps Option)
    pub fn with_statement_ids(mut self, value: Vec<String>) -> Self {
        self.statement_ids = Some(value);
        self
    }
}

impl Default for RemoveDrgRouteDistributionStatementsDetails {
    fn default() -> Self {
        Self::new()
    }
}
