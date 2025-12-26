use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Oracle offers the ability to Bring Your Own IP (BYOIP), importing public IP addresses or IPv6 addresses that you currently own to Oracle Cloud Infrastructure. A {@code ByoipRange} resource is a record of the imported address block (a BYOIP CIDR block) and also some associated metadata. The process used to [Bring Your Own IP](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOIP.htm) is explained in the documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipRange {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the BYOIP CIDR block.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource.
    pub id: String,

    /// The {@code ByoipRange} resource's current state.
    pub lifecycle_state: ByoipRangeLifecycleState,

    /// The date and time the {@code ByoipRange} resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The validation token is an internally-generated ASCII string used in the validation process. See [Importing a CIDR block](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOIP.htm#import_cidr) for details.
    pub validation_token: String,

    /// A list of {@code ByoipRangeVcnIpv6AllocationSummary} objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoip_range_vcn_ipv6_allocations: Option<Vec<ByoipRangeVcnIpv6AllocationSummary>>,

    /// The public IPv4 CIDR block being imported from on-premises to the Oracle cloud.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_asn: Option<ByoipRangeOriginAsn>,

    /// The IPv6 prefix being imported to the Oracle cloud. This prefix must be /48 or larger, and can be subdivided into sub-ranges used across multiple VCNs. A BYOIPv6 prefix can be also assigned across multiple VCNs, and each VCN must be /64 or larger. You may specify a ULA or private IPv6 prefix of /64 or larger to use in the VCN. IPv6-enabled subnets will remain a fixed /64 in size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The {@code ByoipRange} resource's current status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<ByoipRangeLifecycleDetails>,

    /// The date and time the {@code ByoipRange} resource was validated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_validated: Option<DateTime<Utc>>,

    /// The date and time the {@code ByoipRange} resource was advertised to the internet by BGP, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_advertised: Option<DateTime<Utc>>,

    /// The date and time the {@code ByoipRange} resource was withdrawn from advertisement by BGP to the internet, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_withdrawn: Option<DateTime<Utc>>,
}

/// Required fields for ByoipRange
pub struct ByoipRangeRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the BYOIP CIDR block.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource.
    pub id: String,

    /// The {@code ByoipRange} resource's current state.
    pub lifecycle_state: ByoipRangeLifecycleState,

    /// The date and time the {@code ByoipRange} resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The validation token is an internally-generated ASCII string used in the validation process. See [Importing a CIDR block](https://docs.oracle.com/iaas/Content/Network/Concepts/BYOIP.htm#import_cidr) for details.
    pub validation_token: String,
}

impl ByoipRange {
    /// Create a new ByoipRange with required fields
    pub fn new(required: ByoipRangeRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            validation_token: required.validation_token,

            byoip_range_vcn_ipv6_allocations: None,

            cidr_block: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            origin_asn: None,

            ipv6_cidr_block: None,

            lifecycle_details: None,

            time_validated: None,

            time_advertised: None,

            time_withdrawn: None,
        }
    }

    /// Set byoip_range_vcn_ipv6_allocations
    pub fn set_byoip_range_vcn_ipv6_allocations(
        mut self,
        value: Option<Vec<ByoipRangeVcnIpv6AllocationSummary>>,
    ) -> Self {
        self.byoip_range_vcn_ipv6_allocations = value;
        self
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
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

    /// Set origin_asn
    pub fn set_origin_asn(mut self, value: Option<ByoipRangeOriginAsn>) -> Self {
        self.origin_asn = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_cidr_block = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<ByoipRangeLifecycleDetails>) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ByoipRangeLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_validated
    pub fn set_time_validated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_validated = value;
        self
    }

    /// Set time_advertised
    pub fn set_time_advertised(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_advertised = value;
        self
    }

    /// Set time_withdrawn
    pub fn set_time_withdrawn(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_withdrawn = value;
        self
    }

    /// Set validation_token
    pub fn set_validation_token(mut self, value: String) -> Self {
        self.validation_token = value;
        self
    }

    /// Set byoip_range_vcn_ipv6_allocations (unwraps Option)
    pub fn with_byoip_range_vcn_ipv6_allocations(
        mut self,
        value: Vec<ByoipRangeVcnIpv6AllocationSummary>,
    ) -> Self {
        self.byoip_range_vcn_ipv6_allocations = Some(value);
        self
    }

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
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

    /// Set origin_asn (unwraps Option)
    pub fn with_origin_asn(mut self, value: ByoipRangeOriginAsn) -> Self {
        self.origin_asn = Some(value);
        self
    }

    /// Set ipv6_cidr_block (unwraps Option)
    pub fn with_ipv6_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_cidr_block = Some(value.into());
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: ByoipRangeLifecycleDetails) -> Self {
        self.lifecycle_details = Some(value);
        self
    }

    /// Set time_validated (unwraps Option)
    pub fn with_time_validated(mut self, value: DateTime<Utc>) -> Self {
        self.time_validated = Some(value);
        self
    }

    /// Set time_advertised (unwraps Option)
    pub fn with_time_advertised(mut self, value: DateTime<Utc>) -> Self {
        self.time_advertised = Some(value);
        self
    }

    /// Set time_withdrawn (unwraps Option)
    pub fn with_time_withdrawn(mut self, value: DateTime<Utc>) -> Self {
        self.time_withdrawn = Some(value);
        self
    }
}
