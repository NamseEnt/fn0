use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines a module stream profile provide by a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStreamProfile {
    /// The name of the module that contains the stream profile.
    pub module_name: String,

    /// The name of the stream that contains the profile.
    pub stream_name: String,

    /// The name of the profile.
    pub name: String,

    /// A list of packages that constitute the profile.  Each element in the list is the name of a package.  The name is suitable to use as an argument to other OS Management Hub APIs that interact directly with packages.
    pub packages: Vec<String>,

    /// Indicates if this profile is the default for its module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// A description of the contents of the module stream profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Required fields for ModuleStreamProfile
pub struct ModuleStreamProfileRequired {
    /// The name of the module that contains the stream profile.
    pub module_name: String,

    /// The name of the stream that contains the profile.
    pub stream_name: String,

    /// The name of the profile.
    pub name: String,

    /// A list of packages that constitute the profile.  Each element in the list is the name of a package.  The name is suitable to use as an argument to other OS Management Hub APIs that interact directly with packages.
    pub packages: Vec<String>,
}

impl ModuleStreamProfile {
    /// Create a new ModuleStreamProfile with required fields
    pub fn new(required: ModuleStreamProfileRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: required.stream_name,

            name: required.name,

            packages: required.packages,

            is_default: None,

            description: None,
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

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Vec<String>) -> Self {
        self.packages = value;
        self
    }

    /// Set is_default (unwraps Option)
    pub fn with_is_default(mut self, value: bool) -> Self {
        self.is_default = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}
