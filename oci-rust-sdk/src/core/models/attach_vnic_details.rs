use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachVnicDetails {
    pub create_vnic_details: CreateVnicDetails,

    /// The OCID of the instance.
    pub instance_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Which physical network interface card (NIC) the VNIC will use. Defaults to 0. Certain bare metal instance shapes have two active physical NICs (0 and 1). If you add a secondary VNIC to one of these instances, you can specify which NIC the VNIC will use. For more information, see [Virtual Network Interface Cards (VNICs)](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVNICs.htm). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nic_index: Option<i64>,
}

/// Required fields for AttachVnicDetails
pub struct AttachVnicDetailsRequired {
    pub create_vnic_details: CreateVnicDetails,

    /// The OCID of the instance.
    pub instance_id: String,
}

impl AttachVnicDetails {
    /// Create a new AttachVnicDetails with required fields
    pub fn new(required: AttachVnicDetailsRequired) -> Self {
        Self {
            create_vnic_details: required.create_vnic_details,

            instance_id: required.instance_id,

            display_name: None,

            nic_index: None,
        }
    }

    /// Set create_vnic_details
    pub fn set_create_vnic_details(mut self, value: CreateVnicDetails) -> Self {
        self.create_vnic_details = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set nic_index
    pub fn set_nic_index(mut self, value: Option<i64>) -> Self {
        self.nic_index = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set nic_index (unwraps Option)
    pub fn with_nic_index(mut self, value: i64) -> Self {
        self.nic_index = Some(value);
        self
    }
}
