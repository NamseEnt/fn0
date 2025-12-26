use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Update a CrossConnect
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCrossConnectDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Set to true to activate the cross-connect. You activate it after the physical cabling is complete, and you've confirmed the cross-connect's light levels are good and your side of the interface is up. Activation indicates to Oracle that the physical connection is ready. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// A reference name or identifier for the physical fiber connection this cross-connect uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub macsec_properties: Option<UpdateMacsecProperties>,
}

impl UpdateCrossConnectDetails {
    /// Create a new UpdateCrossConnectDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            is_active: None,

            customer_reference_name: None,

            macsec_properties: None,
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

    /// Set is_active
    pub fn set_is_active(mut self, value: Option<bool>) -> Self {
        self.is_active = value;
        self
    }

    /// Set customer_reference_name
    pub fn set_customer_reference_name(mut self, value: Option<String>) -> Self {
        self.customer_reference_name = value;
        self
    }

    /// Set macsec_properties
    pub fn set_macsec_properties(mut self, value: Option<UpdateMacsecProperties>) -> Self {
        self.macsec_properties = value;
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

    /// Set is_active (unwraps Option)
    pub fn with_is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Set customer_reference_name (unwraps Option)
    pub fn with_customer_reference_name(mut self, value: impl Into<String>) -> Self {
        self.customer_reference_name = Some(value.into());
        self
    }

    /// Set macsec_properties (unwraps Option)
    pub fn with_macsec_properties(mut self, value: UpdateMacsecProperties) -> Self {
        self.macsec_properties = Some(value);
        self
    }
}

impl Default for UpdateCrossConnectDetails {
    fn default() -> Self {
        Self::new()
    }
}
