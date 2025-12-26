use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// These details can be included in a request to update a capture filter. A capture filter contains a set of rules governing what traffic a VTAP mirrors or a VCN flow log collects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCaptureFilterDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The set of rules governing what traffic a VTAP mirrors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vtap_capture_filter_rules: Option<Vec<VtapCaptureFilterRuleDetails>>,

    /// The set of rules governing what traffic the VCN flow log collects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_log_capture_filter_rules: Option<Vec<FlowLogCaptureFilterRuleDetails>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

impl UpdateCaptureFilterDetails {
    /// Create a new UpdateCaptureFilterDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            vtap_capture_filter_rules: None,

            flow_log_capture_filter_rules: None,

            freeform_tags: None,
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

    /// Set vtap_capture_filter_rules
    pub fn set_vtap_capture_filter_rules(
        mut self,
        value: Option<Vec<VtapCaptureFilterRuleDetails>>,
    ) -> Self {
        self.vtap_capture_filter_rules = value;
        self
    }

    /// Set flow_log_capture_filter_rules
    pub fn set_flow_log_capture_filter_rules(
        mut self,
        value: Option<Vec<FlowLogCaptureFilterRuleDetails>>,
    ) -> Self {
        self.flow_log_capture_filter_rules = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set vtap_capture_filter_rules (unwraps Option)
    pub fn with_vtap_capture_filter_rules(
        mut self,
        value: Vec<VtapCaptureFilterRuleDetails>,
    ) -> Self {
        self.vtap_capture_filter_rules = Some(value);
        self
    }

    /// Set flow_log_capture_filter_rules (unwraps Option)
    pub fn with_flow_log_capture_filter_rules(
        mut self,
        value: Vec<FlowLogCaptureFilterRuleDetails>,
    ) -> Self {
        self.flow_log_capture_filter_rules = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }
}

impl Default for UpdateCaptureFilterDetails {
    fn default() -> Self {
        Self::new()
    }
}
