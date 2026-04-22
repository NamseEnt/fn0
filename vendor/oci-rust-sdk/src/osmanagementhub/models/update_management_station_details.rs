use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update the management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManagementStationDetails {
    /// User-friendly name for the management station. Does not have to be unique. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified description of the management station. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Hostname of the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// When enabled, the station setup script automatically runs to configure the firewall and SELinux settings on the station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_config_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<UpdateProxyConfigurationDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<UpdateMirrorConfigurationDetails>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

impl UpdateManagementStationDetails {
    /// Create a new UpdateManagementStationDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            description: None,

            hostname: None,

            is_auto_config_enabled: None,

            proxy: None,

            mirror: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set hostname
    pub fn set_hostname(mut self, value: Option<String>) -> Self {
        self.hostname = value;
        self
    }

    /// Set is_auto_config_enabled
    pub fn set_is_auto_config_enabled(mut self, value: Option<bool>) -> Self {
        self.is_auto_config_enabled = value;
        self
    }

    /// Set proxy
    pub fn set_proxy(mut self, value: Option<UpdateProxyConfigurationDetails>) -> Self {
        self.proxy = value;
        self
    }

    /// Set mirror
    pub fn set_mirror(mut self, value: Option<UpdateMirrorConfigurationDetails>) -> Self {
        self.mirror = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set hostname (unwraps Option)
    pub fn with_hostname(mut self, value: impl Into<String>) -> Self {
        self.hostname = Some(value.into());
        self
    }

    /// Set is_auto_config_enabled (unwraps Option)
    pub fn with_is_auto_config_enabled(mut self, value: bool) -> Self {
        self.is_auto_config_enabled = Some(value);
        self
    }

    /// Set proxy (unwraps Option)
    pub fn with_proxy(mut self, value: UpdateProxyConfigurationDetails) -> Self {
        self.proxy = Some(value);
        self
    }

    /// Set mirror (unwraps Option)
    pub fn with_mirror(mut self, value: UpdateMirrorConfigurationDetails) -> Self {
        self.mirror = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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
}

impl Default for UpdateManagementStationDetails {
    fn default() -> Self {
        Self::new()
    }
}
