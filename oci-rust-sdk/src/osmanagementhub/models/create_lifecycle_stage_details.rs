use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLifecycleStageDetails {
    /// A user-friendly name for the lifecycle stage. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// User-specified rank for the lifecycle stage. Rank determines the hierarchy of the lifecycle stages within the lifecycle environment. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub rank: i64,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateLifecycleStageDetails
pub struct CreateLifecycleStageDetailsRequired {
    /// A user-friendly name for the lifecycle stage. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// User-specified rank for the lifecycle stage. Rank determines the hierarchy of the lifecycle stages within the lifecycle environment. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub rank: i64,
}

impl CreateLifecycleStageDetails {
    /// Create a new CreateLifecycleStageDetails with required fields
    pub fn new(required: CreateLifecycleStageDetailsRequired) -> Self {
        Self {
            display_name: required.display_name,

            rank: required.rank,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set rank
    pub fn set_rank(mut self, value: i64) -> Self {
        self.rank = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }
}
