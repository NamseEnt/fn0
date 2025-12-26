use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A capture filter contains a set of *{@link #captureFilterRuleDetails(CaptureFilterRuleDetailsRequest) captureFilterRuleDetails}* governing what traffic is mirrored for a *{@link Vtap}* or captured for a *[VCN Flow Log](https://docs.oracle.com/iaas/Content/Network/Concepts/vcn-flow-logs.htm)*. The capture filter is created with no rules defined, and it must have at least one rule to mirror traffic for the VTAP or collect VCN flow logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureFilter {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the capture filter.
    pub compartment_id: String,

    /// The capture filter's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The capture filter's current administrative state.
    pub lifecycle_state: CaptureFilterLifecycleState,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Indicates which service will use this capture filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_type: Option<CaptureFilterFilterType>,

    /// The date and time the capture filter was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2021-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The set of rules governing what traffic a VTAP mirrors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vtap_capture_filter_rules: Option<Vec<VtapCaptureFilterRuleDetails>>,

    /// The set of rules governing what traffic the VCN flow log collects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_log_capture_filter_rules: Option<Vec<FlowLogCaptureFilterRuleDetails>>,
}

/// Required fields for CaptureFilter
pub struct CaptureFilterRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the capture filter.
    pub compartment_id: String,

    /// The capture filter's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The capture filter's current administrative state.
    pub lifecycle_state: CaptureFilterLifecycleState,
}

impl CaptureFilter {
    /// Create a new CaptureFilter with required fields
    pub fn new(required: CaptureFilterRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            filter_type: None,

            time_created: None,

            vtap_capture_filter_rules: None,

            flow_log_capture_filter_rules: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: CaptureFilterLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set filter_type
    pub fn set_filter_type(mut self, value: Option<CaptureFilterFilterType>) -> Self {
        self.filter_type = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
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

    /// Set filter_type (unwraps Option)
    pub fn with_filter_type(mut self, value: CaptureFilterFilterType) -> Self {
        self.filter_type = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
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
}
