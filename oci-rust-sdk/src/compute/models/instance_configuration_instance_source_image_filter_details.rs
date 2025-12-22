use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Criteria for selecting an image. This is required if imageId is not specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceImageFilterDetails {
    /// The OCID of the compartment containing images to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Filter based on these defined tags. Each key is predefined and scoped to a namespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags_filter: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The image's operating system.
    /// Example: Oracle Linux
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,

    /// The image's operating system version.
    /// Example: 7.2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system_version: Option<String>,
}

impl InstanceConfigurationInstanceSourceImageFilterDetails {
    pub fn new() -> Self {
        Self {
            compartment_id: None,
            defined_tags_filter: None,
            operating_system: None,
            operating_system_version: None,
        }
    }

    pub fn with_compartment_id(mut self, compartment_id: impl Into<String>) -> Self {
        self.compartment_id = Some(compartment_id.into());
        self
    }

    pub fn with_defined_tags_filter(
        mut self,
        filter: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags_filter = Some(filter);
        self
    }

    pub fn with_operating_system(mut self, os: impl Into<String>) -> Self {
        self.operating_system = Some(os.into());
        self
    }

    pub fn with_operating_system_version(mut self, version: impl Into<String>) -> Self {
        self.operating_system_version = Some(version.into());
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, compartment_id: Option<String>) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    /// Set defined_tags_filter
    pub fn set_defined_tags_filter(mut self, defined_tags_filter: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags_filter = defined_tags_filter;
        self
    }

    /// Set operating_system
    pub fn set_operating_system(mut self, operating_system: Option<String>) -> Self {
        self.operating_system = operating_system;
        self
    }

    /// Set operating_system_version
    pub fn set_operating_system_version(mut self, operating_system_version: Option<String>) -> Self {
        self.operating_system_version = operating_system_version;
        self
    }
}

impl Default for InstanceConfigurationInstanceSourceImageFilterDetails {
    fn default() -> Self {
        Self::new()
    }
}
