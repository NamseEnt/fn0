use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a lifecycle environment. A lifecycle environment is a user-defined pipeline to deliver curated, versioned content in a prescribed, methodical manner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLifecycleEnvironmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle environment.
    pub compartment_id: String,

    /// A user-friendly name for the lifecycle environment. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// User-specified list of ranked lifecycle stages used within the lifecycle environment.
    pub stages: Vec<CreateLifecycleStageDetails>,

    /// The CPU architecture of the managed instances in the lifecycle environment.
    pub arch_type: ArchType,

    /// The operating system of the managed instances in the lifecycle environment.
    pub os_family: OsFamily,

    /// The vendor of the operating system used by the managed instances in the lifecycle environment.
    pub vendor_name: VendorName,

    /// User-specified information about the lifecycle environment. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The location of managed instances attached to the lifecycle environment. If no location is provided, the default is 'ON_PREMISE.'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateLifecycleEnvironmentDetails
pub struct CreateLifecycleEnvironmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the lifecycle environment.
    pub compartment_id: String,

    /// A user-friendly name for the lifecycle environment. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// User-specified list of ranked lifecycle stages used within the lifecycle environment.
    pub stages: Vec<CreateLifecycleStageDetails>,

    /// The CPU architecture of the managed instances in the lifecycle environment.
    pub arch_type: ArchType,

    /// The operating system of the managed instances in the lifecycle environment.
    pub os_family: OsFamily,

    /// The vendor of the operating system used by the managed instances in the lifecycle environment.
    pub vendor_name: VendorName,
}

impl CreateLifecycleEnvironmentDetails {
    /// Create a new CreateLifecycleEnvironmentDetails with required fields
    pub fn new(required: CreateLifecycleEnvironmentDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            stages: required.stages,

            arch_type: required.arch_type,

            os_family: required.os_family,

            vendor_name: required.vendor_name,

            description: None,

            location: None,

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

    /// Set stages
    pub fn set_stages(mut self, value: Vec<CreateLifecycleStageDetails>) -> Self {
        self.stages = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
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

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
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
