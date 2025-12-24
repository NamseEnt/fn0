use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The match criteria in a route distribution statement. The match criteria outlines which routes should be imported or exported.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRouteDistributionMatchCriteria {
    pub match_type: String,
}

/// Required fields for DrgRouteDistributionMatchCriteria
pub struct DrgRouteDistributionMatchCriteriaRequired {
    pub match_type: String,
}

impl DrgRouteDistributionMatchCriteria {
    /// Create a new DrgRouteDistributionMatchCriteria with required fields
    pub fn new(required: DrgRouteDistributionMatchCriteriaRequired) -> Self {
        Self {
            match_type: required.match_type,
        }
    }

    /// Set match_type
    pub fn set_match_type(mut self, value: String) -> Self {
        self.match_type = value;
        self
    }
}
