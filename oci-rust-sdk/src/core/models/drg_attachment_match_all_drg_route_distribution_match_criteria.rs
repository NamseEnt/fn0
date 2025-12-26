use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// All routes are imported or exported.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachmentMatchAllDrgRouteDistributionMatchCriteria {
    pub match_type: String,
}

/// Required fields for DrgAttachmentMatchAllDrgRouteDistributionMatchCriteria
pub struct DrgAttachmentMatchAllDrgRouteDistributionMatchCriteriaRequired {
    pub match_type: String,
}

impl DrgAttachmentMatchAllDrgRouteDistributionMatchCriteria {
    /// Create a new DrgAttachmentMatchAllDrgRouteDistributionMatchCriteria with required fields
    pub fn new(required: DrgAttachmentMatchAllDrgRouteDistributionMatchCriteriaRequired) -> Self {
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
