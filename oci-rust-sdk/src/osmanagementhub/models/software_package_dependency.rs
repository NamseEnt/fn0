use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Identifies the dependency for a software package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackageDependency {
    /// The software package's dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency: Option<String>,

    /// The type of the dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_type: Option<String>,

    /// The modifier for the dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_modifier: Option<String>,
}

impl SoftwarePackageDependency {
    /// Create a new SoftwarePackageDependency
    pub fn new() -> Self {
        Self {
            dependency: None,

            dependency_type: None,

            dependency_modifier: None,
        }
    }

    /// Set dependency
    pub fn set_dependency(mut self, value: Option<String>) -> Self {
        self.dependency = value;
        self
    }

    /// Set dependency_type
    pub fn set_dependency_type(mut self, value: Option<String>) -> Self {
        self.dependency_type = value;
        self
    }

    /// Set dependency_modifier
    pub fn set_dependency_modifier(mut self, value: Option<String>) -> Self {
        self.dependency_modifier = value;
        self
    }

    /// Set dependency (unwraps Option)
    pub fn with_dependency(mut self, value: impl Into<String>) -> Self {
        self.dependency = Some(value.into());
        self
    }

    /// Set dependency_type (unwraps Option)
    pub fn with_dependency_type(mut self, value: impl Into<String>) -> Self {
        self.dependency_type = Some(value.into());
        self
    }

    /// Set dependency_modifier (unwraps Option)
    pub fn with_dependency_modifier(mut self, value: impl Into<String>) -> Self {
        self.dependency_modifier = Some(value.into());
        self
    }
}

impl Default for SoftwarePackageDependency {
    fn default() -> Self {
        Self::new()
    }
}
