use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the summary information about a module on a managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupModuleSummary {
    /// The name of the module.
    pub name: String,

    /// The name of the module stream that is enabled for the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_stream: Option<String>,

    /// The list of installed profiles under the currently enabled module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_profiles: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that provides this module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for ManagedInstanceGroupModuleSummary
pub struct ManagedInstanceGroupModuleSummaryRequired {
    /// The name of the module.
    pub name: String,
}

impl ManagedInstanceGroupModuleSummary {
    /// Create a new ManagedInstanceGroupModuleSummary with required fields
    pub fn new(required: ManagedInstanceGroupModuleSummaryRequired) -> Self {
        Self {
            name: required.name,

            enabled_stream: None,

            installed_profiles: None,

            software_source_id: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set enabled_stream
    pub fn set_enabled_stream(mut self, value: Option<String>) -> Self {
        self.enabled_stream = value;
        self
    }

    /// Set installed_profiles
    pub fn set_installed_profiles(mut self, value: Option<Vec<String>>) -> Self {
        self.installed_profiles = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set enabled_stream (unwraps Option)
    pub fn with_enabled_stream(mut self, value: impl Into<String>) -> Self {
        self.enabled_stream = Some(value.into());
        self
    }

    /// Set installed_profiles (unwraps Option)
    pub fn with_installed_profiles(mut self, value: Vec<String>) -> Self {
        self.installed_profiles = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
