use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManagementStationDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// User-friendly name for the management station. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,

    pub proxy: CreateProxyConfigurationDetails,

    pub mirror: CreateMirrorConfigurationDetails,

    /// User-specified description of the management station. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When enabled, the station setup script automatically runs to configure the firewall and SELinux settings on the station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_config_enabled: Option<bool>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateManagementStationDetails
pub struct CreateManagementStationDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the management station.
    pub compartment_id: String,

    /// User-friendly name for the management station. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// Hostname of the management station.
    pub hostname: String,

    pub proxy: CreateProxyConfigurationDetails,

    pub mirror: CreateMirrorConfigurationDetails,
}

impl CreateManagementStationDetails {
    /// Create a new CreateManagementStationDetails with required fields
    pub fn new(required: CreateManagementStationDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            hostname: required.hostname,

            proxy: required.proxy,

            mirror: required.mirror,

            description: None,

            is_auto_config_enabled: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set hostname
    pub fn set_hostname(mut self, value: String) -> Self {
        self.hostname = value;
        self
    }

    /// Set is_auto_config_enabled
    pub fn set_is_auto_config_enabled(mut self, value: Option<bool>) -> Self {
        self.is_auto_config_enabled = value;
        self
    }

    /// Set proxy
    pub fn set_proxy(mut self, value: CreateProxyConfigurationDetails) -> Self {
        self.proxy = value;
        self
    }

    /// Set mirror
    pub fn set_mirror(mut self, value: CreateMirrorConfigurationDetails) -> Self {
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

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set is_auto_config_enabled (unwraps Option)
    pub fn with_is_auto_config_enabled(mut self, value: bool) -> Self {
        self.is_auto_config_enabled = Some(value);
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
