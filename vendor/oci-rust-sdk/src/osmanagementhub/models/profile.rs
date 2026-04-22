use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Object that defines the registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,

    /// A user-friendly name for the profile.
    pub display_name: String,

    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,

    /// The description of the registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station to associate with an instance once registered. Management stations are only used by non-OCI instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_station_id: Option<String>,

    /// The time the registration profile was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The time the registration profile was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modified: Option<DateTime<Utc>>,

    /// The version of the profile. The version is automatically incremented each time the profiled is edited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_version: Option<String>,

    /// The current state of the registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ProfileLifecycleState>,

    /// The type of instance to register.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_type: Option<ProfileRegistrationType>,

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

/// Required fields for Profile
pub struct ProfileRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the registration profile.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,

    /// A user-friendly name for the profile.
    pub display_name: String,

    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,
}

impl Profile {
    /// Create a new Profile with required fields
    pub fn new(required: ProfileRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            vendor_name: required.vendor_name,

            os_family: required.os_family,

            arch_type: required.arch_type,

            profile_type: required.profile_type,

            description: None,

            management_station_id: None,

            time_created: None,

            time_modified: None,

            profile_version: None,

            lifecycle_state: None,

            registration_type: None,

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

    /// Set management_station_id
    pub fn set_management_station_id(mut self, value: Option<String>) -> Self {
        self.management_station_id = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_modified
    pub fn set_time_modified(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_modified = value;
        self
    }

    /// Set profile_version
    pub fn set_profile_version(mut self, value: Option<String>) -> Self {
        self.profile_version = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<ProfileLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set registration_type
    pub fn set_registration_type(mut self, value: Option<ProfileRegistrationType>) -> Self {
        self.registration_type = value;
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

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
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

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set time_modified (unwraps Option)
    pub fn with_time_modified(mut self, value: DateTime<Utc>) -> Self {
        self.time_modified = Some(value);
        self
    }

    /// Set profile_version (unwraps Option)
    pub fn with_profile_version(mut self, value: impl Into<String>) -> Self {
        self.profile_version = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: ProfileLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set registration_type (unwraps Option)
    pub fn with_registration_type(mut self, value: ProfileRegistrationType) -> Self {
        self.registration_type = Some(value);
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
