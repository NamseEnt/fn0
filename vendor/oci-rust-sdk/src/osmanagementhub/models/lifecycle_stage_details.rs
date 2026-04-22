use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides identifying information for the specified lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleStageDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage.
    pub id: String,

    /// Lifecycle stage name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for LifecycleStageDetails
pub struct LifecycleStageDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage.
    pub id: String,
}

impl LifecycleStageDetails {
    /// Create a new LifecycleStageDetails with required fields
    pub fn new(required: LifecycleStageDetailsRequired) -> Self {
        Self {
            id: required.id,

            display_name: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
