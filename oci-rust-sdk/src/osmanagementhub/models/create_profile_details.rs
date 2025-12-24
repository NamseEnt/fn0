use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a new registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileDetails {
    /// A user-friendly name. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,

    pub profile_type: String,

    /// User-specified description of the registration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// description: The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the management station to associate with an instance once registered. This is required when creating a profile for non-OCI instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_station_id: Option<String>,

    /// The type of instance to register.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_type: Option<String>,

    /// Indicates if the profile is set as the default. There is exactly one default profile for a specified architecture, OS family, registration type, and vendor. When registering an instance with the corresonding characteristics, the default profile is used, unless another profile is specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_profile: Option<bool>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateProfileDetails
pub struct CreateProfileDetailsRequired {
    /// A user-friendly name. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the registration profile.
    pub compartment_id: String,

    pub profile_type: String,
}

impl CreateProfileDetails {
    /// Create a new CreateProfileDetails with required fields
    pub fn new(required: CreateProfileDetailsRequired) -> Self {
        Self {
            display_name: required.display_name,

            compartment_id: required.compartment_id,

            profile_type: required.profile_type,

            description: None,

            management_station_id: None,

            registration_type: None,

            is_default_profile: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set registration_type
    pub fn set_registration_type(mut self, value: Option<String>) -> Self {
        self.registration_type = value;
        self
    }

    /// Set is_default_profile
    pub fn set_is_default_profile(mut self, value: Option<bool>) -> Self {
        self.is_default_profile = value;
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

    /// Set registration_type (unwraps Option)
    pub fn with_registration_type(mut self, value: impl Into<String>) -> Self {
        self.registration_type = Some(value.into());
        self
    }

    /// Set is_default_profile (unwraps Option)
    pub fn with_is_default_profile(mut self, value: bool) -> Self {
        self.is_default_profile = Some(value);
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
