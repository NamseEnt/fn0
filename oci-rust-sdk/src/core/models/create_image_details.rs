use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Either instanceId or imageSourceDetails must be provided in addition to other required parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateImageDetails {
    /// The OCID of the compartment you want the image to be created in.
    pub compartment_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name for the image. It does not have to be unique, and it's changeable. Avoid entering confidential information. <p> You cannot use a platform image name as a custom image name. <p> Example: {@code My Oracle Linux image}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_source_details: Option<ImageSourceViaObjectStorageTupleDetails>,

    /// The OCID of the instance you want to use as the basis for the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// Specifies the configuration mode for launching virtual machine (VM) instances. The configuration modes are: * {@code NATIVE} - VM instances launch with iSCSI boot and VFIO devices. The default value for platform images. * {@code EMULATED} - VM instances launch with emulated devices, such as the E1000 network driver and emulated SCSI disk controller. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers. * {@code CUSTOM} - VM instances launch with custom configuration settings specified in the {@code LaunchOptions} parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<CreateImageDetailsLaunchMode>,
}

/// Required fields for CreateImageDetails
pub struct CreateImageDetailsRequired {
    /// The OCID of the compartment you want the image to be created in.
    pub compartment_id: String,
}

impl CreateImageDetails {
    /// Create a new CreateImageDetails with required fields
    pub fn new(required: CreateImageDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            image_source_details: None,

            instance_id: None,

            launch_mode: None,
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

    /// Set image_source_details
    pub fn set_image_source_details(
        mut self,
        value: Option<ImageSourceViaObjectStorageTupleDetails>,
    ) -> Self {
        self.image_source_details = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set launch_mode
    pub fn set_launch_mode(mut self, value: Option<CreateImageDetailsLaunchMode>) -> Self {
        self.launch_mode = value;
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

    /// Set image_source_details (unwraps Option)
    pub fn with_image_source_details(
        mut self,
        value: ImageSourceViaObjectStorageTupleDetails,
    ) -> Self {
        self.image_source_details = Some(value);
        self
    }

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set launch_mode (unwraps Option)
    pub fn with_launch_mode(mut self, value: CreateImageDetailsLaunchMode) -> Self {
        self.launch_mode = Some(value);
        self
    }
}
