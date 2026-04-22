use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details about a specific appstream module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSpecDetails {
    /// Name of the module.
    pub name: String,

    /// The stream of the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,

    /// The module profile to be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

/// Required fields for ModuleSpecDetails
pub struct ModuleSpecDetailsRequired {
    /// Name of the module.
    pub name: String,
}

impl ModuleSpecDetails {
    /// Create a new ModuleSpecDetails with required fields
    pub fn new(required: ModuleSpecDetailsRequired) -> Self {
        Self {
            name: required.name,

            stream: None,

            profile: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set stream
    pub fn set_stream(mut self, value: Option<String>) -> Self {
        self.stream = value;
        self
    }

    /// Set profile
    pub fn set_profile(mut self, value: Option<String>) -> Self {
        self.profile = value;
        self
    }

    /// Set stream (unwraps Option)
    pub fn with_stream(mut self, value: impl Into<String>) -> Self {
        self.stream = Some(value.into());
        self
    }

    /// Set profile (unwraps Option)
    pub fn with_profile(mut self, value: impl Into<String>) -> Self {
        self.profile = Some(value.into());
        self
    }
}
