use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the summary information for a module stream contained within a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamSummary {
    /// The name of the stream.
    pub name: String,

    /// The name of the module that contains the stream.
    pub module_name: String,

    /// List of profiles in the stream.
    pub profiles: Vec<String>,

    /// Indicates whether this module stream is the latest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that contains the the module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for ModuleStreamSummary
pub struct ModuleStreamSummaryRequired {
    /// The name of the stream.
    pub name: String,

    /// The name of the module that contains the stream.
    pub module_name: String,

    /// List of profiles in the stream.
    pub profiles: Vec<String>,
}

impl ModuleStreamSummary {
    /// Create a new ModuleStreamSummary with required fields
    pub fn new(required: ModuleStreamSummaryRequired) -> Self {
        Self {
            name: required.name,

            module_name: required.module_name,

            profiles: required.profiles,

            is_latest: None,

            software_source_id: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set profiles
    pub fn set_profiles(mut self, value: Vec<String>) -> Self {
        self.profiles = value;
        self
    }

    /// Set is_latest
    pub fn set_is_latest(mut self, value: Option<bool>) -> Self {
        self.is_latest = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set is_latest (unwraps Option)
    pub fn with_is_latest(mut self, value: bool) -> Self {
        self.is_latest = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
