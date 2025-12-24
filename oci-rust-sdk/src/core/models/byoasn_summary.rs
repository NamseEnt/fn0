use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Information about a {@code Byoasn} resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoasnSummary {
    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Byoasn} resource.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub id: String,

    /// The {@code Byoasn} resource's current state.
    pub lifecycle_state: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The date and time the {@code Byoasn} resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for ByoasnSummary
pub struct ByoasnSummaryRequired {
    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Byoasn} resource.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub id: String,

    /// The {@code Byoasn} resource's current state.
    pub lifecycle_state: String,
}

impl ByoasnSummary {
    /// Create a new ByoasnSummary with required fields
    pub fn new(required: ByoasnSummaryRequired) -> Self {
        Self {
            asn: required.asn,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            display_name: None,

            defined_tags: None,

            freeform_tags: None,

            system_tags: None,

            time_created: None,
        }
    }

    /// Set asn
    pub fn set_asn(mut self, value: i64) -> Self {
        self.asn = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
