use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Oracle offers the ability to Bring Your Own Autonomous System Number (BYOASN), importing AS Numbers you currently own to Oracle Cloud Infrastructure. A {@code Byoasn} resource is a record of the imported AS Number and also some associated metadata. The process used to [Bring Your Own ASN](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOASN.htm) is explained in the documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Byoasn {
    /// The {@code Byoasn} resource's current state.
    pub lifecycle_state: ByoasnLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub id: String,

    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Byoasn} resource.
    pub compartment_id: String,

    /// The validation token is an internally-generated ASCII string used in the validation process. See [Importing a Byoasn](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOASN.htm) for details.
    pub validation_token: String,

    /// The date and time the {@code Byoasn} resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The date and time the {@code Byoasn} resource was validated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_validated: Option<DateTime<Utc>>,

    /// The date and time the {@code Byoasn} resource was last updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// The BYOIP Ranges that has the {@code Byoasn} as origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoip_ranges: Option<Vec<ByoasnByoipRange>>,
}

/// Required fields for Byoasn
pub struct ByoasnRequired {
    /// The {@code Byoasn} resource's current state.
    pub lifecycle_state: ByoasnLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    pub id: String,

    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Byoasn} resource.
    pub compartment_id: String,

    /// The validation token is an internally-generated ASCII string used in the validation process. See [Importing a Byoasn](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOASN.htm) for details.
    pub validation_token: String,

    /// The date and time the {@code Byoasn} resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl Byoasn {
    /// Create a new Byoasn with required fields
    pub fn new(required: ByoasnRequired) -> Self {
        Self {
            lifecycle_state: required.lifecycle_state,

            id: required.id,

            asn: required.asn,

            compartment_id: required.compartment_id,

            validation_token: required.validation_token,

            time_created: required.time_created,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            time_validated: None,

            time_updated: None,

            byoip_ranges: None,
        }
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ByoasnLifecycleState) -> Self {
        self.lifecycle_state = value;
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

    /// Set time_validated
    pub fn set_time_validated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_validated = value;
        self
    }

    /// Set validation_token
    pub fn set_validation_token(mut self, value: String) -> Self {
        self.validation_token = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set byoip_ranges
    pub fn set_byoip_ranges(mut self, value: Option<Vec<ByoasnByoipRange>>) -> Self {
        self.byoip_ranges = value;
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

    /// Set time_validated (unwraps Option)
    pub fn with_time_validated(mut self, value: DateTime<Utc>) -> Self {
        self.time_validated = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set byoip_ranges (unwraps Option)
    pub fn with_byoip_ranges(mut self, value: Vec<ByoasnByoipRange>) -> Self {
        self.byoip_ranges = Some(value);
        self
    }
}
