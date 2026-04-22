use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about a module stream made available to a managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupAvailableModuleSummary {
    /// The name of the module that is available to the managed instance group.
    pub name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that provides the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for ManagedInstanceGroupAvailableModuleSummary
pub struct ManagedInstanceGroupAvailableModuleSummaryRequired {
    /// The name of the module that is available to the managed instance group.
    pub name: String,
}

impl ManagedInstanceGroupAvailableModuleSummary {
    /// Create a new ManagedInstanceGroupAvailableModuleSummary with required fields
    pub fn new(required: ManagedInstanceGroupAvailableModuleSummaryRequired) -> Self {
        Self {
            name: required.name,

            software_source_id: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
