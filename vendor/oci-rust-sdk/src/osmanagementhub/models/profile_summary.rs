use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,

    /// A user-friendly name for the profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified description of the registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station to associate with an instance once registered. Management stations are only used with non-OCI instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_station_id: Option<String>,

    /// The type of registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_type: Option<ProfileType>,

    /// The type of instance to register.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_type: Option<String>,

    /// The vendor of the operating system for the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<VendorName>,

    /// The operating system family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The architecture type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,

    /// The time the registration profile was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The current state of the registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    /// Indicates if the profile is set as the default. There is exactly one default profile for a specified architecture, OS family, registration type, and vendor. When registering an instance with the corresonding characteristics, the default profile is used, unless another profile is specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_profile: Option<bool>,

    /// Indicates if the profile was created by the service. OS Management Hub provides a limited set of standardized profiles that can be used to register Autonomous Linux or Windows instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_service_provided_profile: Option<bool>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for ProfileSummary
pub struct ProfileSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,
}

impl ProfileSummary {
    /// Create a new ProfileSummary with required fields
    pub fn new(required: ProfileSummaryRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: None,

            description: None,

            management_station_id: None,

            profile_type: None,

            registration_type: None,

            vendor_name: None,

            os_family: None,

            arch_type: None,

            time_created: None,

            lifecycle_state: None,

            is_default_profile: None,

            is_service_provided_profile: None,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
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

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set management_station_id
    pub fn set_management_station_id(mut self, value: Option<String>) -> Self {
        self.management_station_id = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: Option<ProfileType>) -> Self {
        self.profile_type = value;
        self
    }

    /// Set registration_type
    pub fn set_registration_type(mut self, value: Option<String>) -> Self {
        self.registration_type = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: Option<VendorName>) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: Option<OsFamily>) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: Option<ArchType>) -> Self {
        self.arch_type = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<String>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set is_default_profile
    pub fn set_is_default_profile(mut self, value: Option<bool>) -> Self {
        self.is_default_profile = value;
        self
    }

    /// Set is_service_provided_profile
    pub fn set_is_service_provided_profile(mut self, value: Option<bool>) -> Self {
        self.is_service_provided_profile = value;
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

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
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

    /// Set management_station_id (unwraps Option)
    pub fn with_management_station_id(mut self, value: impl Into<String>) -> Self {
        self.management_station_id = Some(value.into());
        self
    }

    /// Set profile_type (unwraps Option)
    pub fn with_profile_type(mut self, value: ProfileType) -> Self {
        self.profile_type = Some(value);
        self
    }

    /// Set registration_type (unwraps Option)
    pub fn with_registration_type(mut self, value: impl Into<String>) -> Self {
        self.registration_type = Some(value.into());
        self
    }

    /// Set vendor_name (unwraps Option)
    pub fn with_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = Some(value);
        self
    }

    /// Set os_family (unwraps Option)
    pub fn with_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = Some(value);
        self
    }

    /// Set arch_type (unwraps Option)
    pub fn with_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_state = Some(value.into());
        self
    }

    /// Set is_default_profile (unwraps Option)
    pub fn with_is_default_profile(mut self, value: bool) -> Self {
        self.is_default_profile = Some(value);
        self
    }

    /// Set is_service_provided_profile (unwraps Option)
    pub fn with_is_service_provided_profile(mut self, value: bool) -> Self {
        self.is_service_provided_profile = Some(value);
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

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }
}
