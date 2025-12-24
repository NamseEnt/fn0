use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A boot disk image for launching an instance. For more information, see [Overview of the Compute Service](https://docs.oracle.com/iaas/Content/Compute/Concepts/computeoverview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The OCID of the compartment containing the instance you want to use as the basis for the image.
    pub compartment_id: String,

    /// Whether instances launched with this image can be used to create new images. For example, you cannot create an image of an Oracle Database instance. <p> Example: {@code true}
    pub create_image_allowed: bool,

    /// The OCID of the image.
    pub id: String,

    pub lifecycle_state: ImageLifecycleState,

    /// The image's operating system. <p> Example: {@code Oracle Linux}
    pub operating_system: String,

    /// The image's operating system version. <p> Example: {@code 7.2}
    pub operating_system_version: String,

    /// The date and time the image was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the image originally used to launch the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_image_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name for the image. It does not have to be unique, and it's changeable. Avoid entering confidential information. <p> You cannot use a platform image name as a custom image name. <p> Example: {@code My custom Oracle Linux image}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Specifies the configuration mode for launching virtual machine (VM) instances. The configuration modes are: * {@code NATIVE} - VM instances launch with iSCSI boot and VFIO devices. The default value for platform images. * {@code EMULATED} - VM instances launch with emulated devices, such as the E1000 network driver and emulated SCSI disk controller. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers. * {@code CUSTOM} - VM instances launch with custom configuration settings specified in the {@code LaunchOptions} parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<ImageLaunchMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_options: Option<LaunchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_features: Option<InstanceAgentFeatures>,

    /// The listing type of the image. The default value is \"NONE\".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_type: Option<ImageListingType>,

    /// The boot volume size for an instance launched from this image (1 MB = 1,048,576 bytes). Note this is not the same as the size of the image when it was exported or the actual size of the image. <p> Example: {@code 47694} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_m_bs: Option<i64>,

    /// The size of the internal storage for this image that is subject to billing (1 GB = 1,073,741,824 bytes). <p> Example: {@code 100} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billable_size_in_g_bs: Option<i64>,
}

/// Required fields for Image
pub struct ImageRequired {
    /// The OCID of the compartment containing the instance you want to use as the basis for the image.
    pub compartment_id: String,

    /// Whether instances launched with this image can be used to create new images. For example, you cannot create an image of an Oracle Database instance. <p> Example: {@code true}
    pub create_image_allowed: bool,

    /// The OCID of the image.
    pub id: String,

    pub lifecycle_state: ImageLifecycleState,

    /// The image's operating system. <p> Example: {@code Oracle Linux}
    pub operating_system: String,

    /// The image's operating system version. <p> Example: {@code 7.2}
    pub operating_system_version: String,

    /// The date and time the image was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl Image {
    /// Create a new Image with required fields
    pub fn new(required: ImageRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            create_image_allowed: required.create_image_allowed,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            operating_system: required.operating_system,

            operating_system_version: required.operating_system_version,

            time_created: required.time_created,

            base_image_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            launch_mode: None,

            launch_options: None,

            agent_features: None,

            listing_type: None,

            size_in_m_bs: None,

            billable_size_in_g_bs: None,
        }
    }

    /// Set base_image_id
    pub fn set_base_image_id(mut self, value: Option<String>) -> Self {
        self.base_image_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set create_image_allowed
    pub fn set_create_image_allowed(mut self, value: bool) -> Self {
        self.create_image_allowed = value;
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

    /// Set launch_mode
    pub fn set_launch_mode(mut self, value: Option<ImageLaunchMode>) -> Self {
        self.launch_mode = value;
        self
    }

    /// Set launch_options
    pub fn set_launch_options(mut self, value: Option<LaunchOptions>) -> Self {
        self.launch_options = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ImageLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set operating_system
    pub fn set_operating_system(mut self, value: String) -> Self {
        self.operating_system = value;
        self
    }

    /// Set operating_system_version
    pub fn set_operating_system_version(mut self, value: String) -> Self {
        self.operating_system_version = value;
        self
    }

    /// Set agent_features
    pub fn set_agent_features(mut self, value: Option<InstanceAgentFeatures>) -> Self {
        self.agent_features = value;
        self
    }

    /// Set listing_type
    pub fn set_listing_type(mut self, value: Option<ImageListingType>) -> Self {
        self.listing_type = value;
        self
    }

    /// Set size_in_m_bs
    pub fn set_size_in_m_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_m_bs = value;
        self
    }

    /// Set billable_size_in_g_bs
    pub fn set_billable_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.billable_size_in_g_bs = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set base_image_id (unwraps Option)
    pub fn with_base_image_id(mut self, value: impl Into<String>) -> Self {
        self.base_image_id = Some(value.into());
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

    /// Set launch_mode (unwraps Option)
    pub fn with_launch_mode(mut self, value: ImageLaunchMode) -> Self {
        self.launch_mode = Some(value);
        self
    }

    /// Set launch_options (unwraps Option)
    pub fn with_launch_options(mut self, value: LaunchOptions) -> Self {
        self.launch_options = Some(value);
        self
    }

    /// Set agent_features (unwraps Option)
    pub fn with_agent_features(mut self, value: InstanceAgentFeatures) -> Self {
        self.agent_features = Some(value);
        self
    }

    /// Set listing_type (unwraps Option)
    pub fn with_listing_type(mut self, value: ImageListingType) -> Self {
        self.listing_type = Some(value);
        self
    }

    /// Set size_in_m_bs (unwraps Option)
    pub fn with_size_in_m_bs(mut self, value: i64) -> Self {
        self.size_in_m_bs = Some(value);
        self
    }

    /// Set billable_size_in_g_bs (unwraps Option)
    pub fn with_billable_size_in_g_bs(mut self, value: i64) -> Self {
        self.billable_size_in_g_bs = Some(value);
        self
    }
}
