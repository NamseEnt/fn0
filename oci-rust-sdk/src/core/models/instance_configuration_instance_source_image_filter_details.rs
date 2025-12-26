use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// These are the criteria for selecting an image. This is required if imageId is not specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceImageFilterDetails {
    /// The OCID of the compartment containing images to search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Filter based on these defined tags. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags_filter: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The image's operating system. <p> Example: {@code Oracle Linux}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,

    /// The image's operating system version. <p> Example: {@code 7.2}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system_version: Option<String>,
}

impl InstanceConfigurationInstanceSourceImageFilterDetails {
    /// Create a new InstanceConfigurationInstanceSourceImageFilterDetails
    pub fn new() -> Self {
        Self {
            compartment_id: None,

            defined_tags_filter: None,

            operating_system: None,

            operating_system_version: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set defined_tags_filter
    pub fn set_defined_tags_filter(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags_filter = value;
        self
    }

    /// Set operating_system
    pub fn set_operating_system(mut self, value: Option<String>) -> Self {
        self.operating_system = value;
        self
    }

    /// Set operating_system_version
    pub fn set_operating_system_version(mut self, value: Option<String>) -> Self {
        self.operating_system_version = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set defined_tags_filter (unwraps Option)
    pub fn with_defined_tags_filter(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags_filter = Some(value);
        self
    }

    /// Set operating_system (unwraps Option)
    pub fn with_operating_system(mut self, value: impl Into<String>) -> Self {
        self.operating_system = Some(value.into());
        self
    }

    /// Set operating_system_version (unwraps Option)
    pub fn with_operating_system_version(mut self, value: impl Into<String>) -> Self {
        self.operating_system_version = Some(value.into());
        self
    }
}

impl Default for InstanceConfigurationInstanceSourceImageFilterDetails {
    fn default() -> Self {
        Self::new()
    }
}
