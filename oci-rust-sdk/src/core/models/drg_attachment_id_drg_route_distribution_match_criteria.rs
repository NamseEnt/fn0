use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The criteria by which a specific attachment will import routes to the DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachmentIdDrgRouteDistributionMatchCriteria {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,

    pub match_type: String,
}

/// Required fields for DrgAttachmentIdDrgRouteDistributionMatchCriteria
pub struct DrgAttachmentIdDrgRouteDistributionMatchCriteriaRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,

    pub match_type: String,
}

impl DrgAttachmentIdDrgRouteDistributionMatchCriteria {
    /// Create a new DrgAttachmentIdDrgRouteDistributionMatchCriteria with required fields
    pub fn new(required: DrgAttachmentIdDrgRouteDistributionMatchCriteriaRequired) -> Self {
        Self {
            drg_attachment_id: required.drg_attachment_id,

            match_type: required.match_type,
        }
    }

    /// Set drg_attachment_id
    pub fn set_drg_attachment_id(mut self, value: String) -> Self {
        self.drg_attachment_id = value;
        self
    }

    /// Set match_type
    pub fn set_match_type(mut self, value: String) -> Self {
        self.match_type = value;
        self
    }
}
