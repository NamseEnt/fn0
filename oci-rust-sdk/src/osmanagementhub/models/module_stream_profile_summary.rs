use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the summary information for a module stream profile contained within a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamProfileSummary {
    /// The name of the module that contains the stream profile.
    pub module_name: String,

    /// The name of the stream that contains the profile.
    pub stream_name: String,

    /// The name of the profile.
    pub name: String,

    /// Indicates if this profile is the default for the module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// Required fields for ModuleStreamProfileSummary
pub struct ModuleStreamProfileSummaryRequired {
    /// The name of the module that contains the stream profile.
    pub module_name: String,

    /// The name of the stream that contains the profile.
    pub stream_name: String,

    /// The name of the profile.
    pub name: String,
}

impl ModuleStreamProfileSummary {
    /// Create a new ModuleStreamProfileSummary with required fields
    pub fn new(required: ModuleStreamProfileSummaryRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: required.stream_name,

            name: required.name,

            is_default: None,
        }
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set stream_name
    pub fn set_stream_name(mut self, value: String) -> Self {
        self.stream_name = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set is_default
    pub fn set_is_default(mut self, value: Option<bool>) -> Self {
        self.is_default = value;
        self
    }

    /// Set is_default (unwraps Option)
    pub fn with_is_default(mut self, value: bool) -> Self {
        self.is_default = Some(value);
        self
    }
}
