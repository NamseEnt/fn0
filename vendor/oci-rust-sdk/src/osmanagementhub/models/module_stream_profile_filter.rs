use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a filter for module streams and profiles from a vendor software source to create or update a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamProfileFilter {
    /// Module name.
    pub module_name: String,

    /// The type of the filter.
    pub filter_type: FilterType,

    /// Profile name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name: Option<String>,

    /// Stream name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_name: Option<String>,
}

/// Required fields for ModuleStreamProfileFilter
pub struct ModuleStreamProfileFilterRequired {
    /// Module name.
    pub module_name: String,

    /// The type of the filter.
    pub filter_type: FilterType,
}

impl ModuleStreamProfileFilter {
    /// Create a new ModuleStreamProfileFilter with required fields
    pub fn new(required: ModuleStreamProfileFilterRequired) -> Self {
        Self {
            module_name: required.module_name,

            filter_type: required.filter_type,

            profile_name: None,

            stream_name: None,
        }
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set profile_name
    pub fn set_profile_name(mut self, value: Option<String>) -> Self {
        self.profile_name = value;
        self
    }

    /// Set stream_name
    pub fn set_stream_name(mut self, value: Option<String>) -> Self {
        self.stream_name = value;
        self
    }

    /// Set filter_type
    pub fn set_filter_type(mut self, value: FilterType) -> Self {
        self.filter_type = value;
        self
    }

    /// Set profile_name (unwraps Option)
    pub fn with_profile_name(mut self, value: impl Into<String>) -> Self {
        self.profile_name = Some(value.into());
        self
    }

    /// Set stream_name (unwraps Option)
    pub fn with_stream_name(mut self, value: impl Into<String>) -> Self {
        self.stream_name = Some(value.into());
        self
    }
}
