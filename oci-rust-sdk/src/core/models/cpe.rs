use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An object you create when setting up a Site-to-Site VPN between your on-premises network and VCN. The {@code Cpe} is a virtual representation of your customer-premises equipment, which is the actual router on-premises at your site at your end of the Site-to-Site VPN IPSec connection. For more information, see [Overview of the Networking Service](https://docs.oracle.com/iaas/Content/Network/Concepts/overview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpe {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the CPE.
    pub compartment_id: String,

    /// The CPE's Oracle ID (OCID).
    pub id: String,

    /// The public IP address of the on-premises router.
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

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the CPE's device type. The Networking service maintains a general list of CPE device types (for example, Cisco ASA). For each type, Oracle provides CPE configuration content that can help a network engineer configure the CPE. The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) uniquely identifies the type of device. To get the OCIDs for the device types on the list, see {@link #listCpeDeviceShapes(ListCpeDeviceShapesRequest) listCpeDeviceShapes}. <p> For information about how to generate CPE configuration content for a CPE device type, see: <p> {@link #getCpeDeviceConfigContent(GetCpeDeviceConfigContentRequest) getCpeDeviceConfigContent} * {@link #getIpsecCpeDeviceConfigContent(GetIpsecCpeDeviceConfigContentRequest) getIpsecCpeDeviceConfigContent} * {@link #getTunnelCpeDeviceConfigContent(GetTunnelCpeDeviceConfigContentRequest) getTunnelCpeDeviceConfigContent} * {@link #getTunnelCpeDeviceConfig(GetTunnelCpeDeviceConfigRequest) getTunnelCpeDeviceConfig}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_device_shape_id: Option<String>,

    /// The date and time the CPE was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// Indicates whether this CPE is of type {@code private} or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

/// Required fields for Cpe
pub struct CpeRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the CPE.
    pub compartment_id: String,

    /// The CPE's Oracle ID (OCID).
    pub id: String,

    /// The public IP address of the on-premises router.
    pub ip_address: String,
}

impl Cpe {
    /// Create a new Cpe with required fields
    pub fn new(required: CpeRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            ip_address: required.ip_address,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            cpe_device_shape_id: None,

            time_created: None,

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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
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

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
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

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set is_private (unwraps Option)
    pub fn with_is_private(mut self, value: bool) -> Self {
        self.is_private = Some(value);
        self
    }
}
