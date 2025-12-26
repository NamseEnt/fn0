use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateImageDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Operating system <p> Example: {@code Oracle Linux}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,

    /// Operating system version <p> Example: {@code 7.4}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system_version: Option<String>,
}

impl UpdateImageDetails {
    /// Create a new UpdateImageDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            operating_system: None,

            operating_system_version: None,
        }
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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

impl Default for UpdateImageDetails {
    fn default() -> Self {
        Self::new()
    }
}
