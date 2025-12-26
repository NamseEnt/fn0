use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCpeDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the CPE.
    pub compartment_id: String,

    /// The public IP address of the on-premises router. <p> Example: {@code 203.0.113.2}
    pub ip_address: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE device type. You can provide a value if you want to later generate CPE device configuration content for IPSec connections that use this CPE. You can also call {@link #updateCpe(UpdateCpeRequest) updateCpe} later to provide a value. For a list of possible values, see {@link #listCpeDeviceShapes(ListCpeDeviceShapesRequest) listCpeDeviceShapes}. <p> For more information about generating CPE device configuration content, see: <p> {@link #getCpeDeviceConfigContent(GetCpeDeviceConfigContentRequest) getCpeDeviceConfigContent} * {@link #getIpsecCpeDeviceConfigContent(GetIpsecCpeDeviceConfigContentRequest) getIpsecCpeDeviceConfigContent} * {@link #getTunnelCpeDeviceConfigContent(GetTunnelCpeDeviceConfigContentRequest) getTunnelCpeDeviceConfigContent} * {@link #getTunnelCpeDeviceConfig(GetTunnelCpeDeviceConfigRequest) getTunnelCpeDeviceConfig}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_device_shape_id: Option<String>,

    /// Indicates whether this CPE is of type {@code private} or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

/// Required fields for CreateCpeDetails
pub struct CreateCpeDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the CPE.
    pub compartment_id: String,

    /// The public IP address of the on-premises router. <p> Example: {@code 203.0.113.2}
    pub ip_address: String,
}

impl CreateCpeDetails {
    /// Create a new CreateCpeDetails with required fields
    pub fn new(required: CreateCpeDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            ip_address: required.ip_address,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            cpe_device_shape_id: None,

            is_private: None,
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

    /// Set ip_address
    pub fn set_ip_address(mut self, value: String) -> Self {
        self.ip_address = value;
        self
    }

    /// Set cpe_device_shape_id
    pub fn set_cpe_device_shape_id(mut self, value: Option<String>) -> Self {
        self.cpe_device_shape_id = value;
        self
    }

    /// Set is_private
    pub fn set_is_private(mut self, value: Option<bool>) -> Self {
        self.is_private = value;
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

    /// Set cpe_device_shape_id (unwraps Option)
    pub fn with_cpe_device_shape_id(mut self, value: impl Into<String>) -> Self {
        self.cpe_device_shape_id = Some(value.into());
        self
    }

    /// Set is_private (unwraps Option)
    pub fn with_is_private(mut self, value: bool) -> Self {
        self.is_private = Some(value);
        self
    }
}
