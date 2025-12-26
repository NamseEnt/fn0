use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The attachment type from which the DRG will import routes. Routes will be imported from all attachments of this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachmentTypeDrgRouteDistributionMatchCriteria {
    /// The type of the network resource to be included in this match. A match for a network type implies that all DRG attachments of that type insert routes into the table.
    pub attachment_type: DrgAttachmentTypeDrgRouteDistributionMatchCriteriaAttachmentType,

    pub match_type: String,
}

/// Required fields for DrgAttachmentTypeDrgRouteDistributionMatchCriteria
pub struct DrgAttachmentTypeDrgRouteDistributionMatchCriteriaRequired {
    /// The type of the network resource to be included in this match. A match for a network type implies that all DRG attachments of that type insert routes into the table.
    pub attachment_type: DrgAttachmentTypeDrgRouteDistributionMatchCriteriaAttachmentType,

    pub match_type: String,
}

impl DrgAttachmentTypeDrgRouteDistributionMatchCriteria {
    /// Create a new DrgAttachmentTypeDrgRouteDistributionMatchCriteria with required fields
    pub fn new(required: DrgAttachmentTypeDrgRouteDistributionMatchCriteriaRequired) -> Self {
        Self {
            attachment_type: required.attachment_type,

            match_type: required.match_type,
        }
    }

    /// Set attachment_type
    pub fn set_attachment_type(
        mut self,
        value: DrgAttachmentTypeDrgRouteDistributionMatchCriteriaAttachmentType,
    ) -> Self {
        self.attachment_type = value;
        self
    }

    /// Set match_type
    pub fn set_match_type(mut self, value: String) -> Self {
        self.match_type = value;
        self
    }
}
