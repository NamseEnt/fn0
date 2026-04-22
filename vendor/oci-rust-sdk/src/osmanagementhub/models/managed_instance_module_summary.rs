use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a module on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceModuleSummary {
    /// The module name.
    pub name: String,

    /// The stream that is enabled in the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_stream: Option<String>,

    /// List of installed profiles in the enabled stream of the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_profiles: Option<Vec<String>>,

    /// List of streams that are active in the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_streams: Option<Vec<String>>,

    /// List of streams that are disabled in the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_streams: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that provides this module and the associated streams.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for ManagedInstanceModuleSummary
pub struct ManagedInstanceModuleSummaryRequired {
    /// The module name.
    pub name: String,
}

impl ManagedInstanceModuleSummary {
    /// Create a new ManagedInstanceModuleSummary with required fields
    pub fn new(required: ManagedInstanceModuleSummaryRequired) -> Self {
        Self {
            name: required.name,

            enabled_stream: None,

            installed_profiles: None,

            active_streams: None,

            disabled_streams: None,

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

    /// Set active_streams
    pub fn set_active_streams(mut self, value: Option<Vec<String>>) -> Self {
        self.active_streams = value;
        self
    }

    /// Set disabled_streams
    pub fn set_disabled_streams(mut self, value: Option<Vec<String>>) -> Self {
        self.disabled_streams = value;
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

    /// Set active_streams (unwraps Option)
    pub fn with_active_streams(mut self, value: Vec<String>) -> Self {
        self.active_streams = Some(value);
        self
    }

    /// Set disabled_streams (unwraps Option)
    pub fn with_disabled_streams(mut self, value: Vec<String>) -> Self {
        self.disabled_streams = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
