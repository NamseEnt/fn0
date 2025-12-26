use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Represents an attachment between a VNIC and an instance. For more information, see [Virtual Network Interface Cards (VNICs)](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVNICs.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnicAttachment {
    /// The availability domain of the instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment the VNIC attachment is in, which is the same compartment the instance is in.
    pub compartment_id: String,

    /// The OCID of the VNIC attachment.
    pub id: String,

    /// The OCID of the instance.
    pub instance_id: String,

    /// The current state of the VNIC attachment.
    pub lifecycle_state: VnicAttachmentLifecycleState,

    /// The date and time the VNIC attachment was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Which physical network interface card (NIC) the VNIC uses. Certain bare metal instance shapes have two active physical NICs (0 and 1). If you add a secondary VNIC to one of these instances, you can specify which NIC the VNIC will use. For more information, see [Virtual Network Interface Cards (VNICs)](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVNICs.htm). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nic_index: Option<i64>,

    /// The OCID of the subnet to create the VNIC in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// The OCID of the VLAN to create the VNIC in. Creating the VNIC in a VLAN (instead of a subnet) is possible only if you are an Oracle Cloud VMware Solution customer. See {@link Vlan}. <p> An error is returned if the instance already has a VNIC attached to it from this VLAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<String>,

    /// The Oracle-assigned VLAN tag of the attached VNIC. Available after the attachment process is complete. <p> However, if the VNIC belongs to a VLAN as part of the Oracle Cloud VMware Solution, the {@code vlanTag} value is instead the value of the {@code vlanTag} attribute for the VLAN. See {@link Vlan}. <p> Example: {@code 0} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_tag: Option<i64>,

    /// The OCID of the VNIC. Available after the attachment process is complete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,
}

/// Required fields for VnicAttachment
pub struct VnicAttachmentRequired {
    /// The availability domain of the instance. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment the VNIC attachment is in, which is the same compartment the instance is in.
    pub compartment_id: String,

    /// The OCID of the VNIC attachment.
    pub id: String,

    /// The OCID of the instance.
    pub instance_id: String,

    /// The current state of the VNIC attachment.
    pub lifecycle_state: VnicAttachmentLifecycleState,

    /// The date and time the VNIC attachment was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl VnicAttachment {
    /// Create a new VnicAttachment with required fields
    pub fn new(required: VnicAttachmentRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            instance_id: required.instance_id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            display_name: None,

            nic_index: None,

            subnet_id: None,

            vlan_id: None,

            vlan_tag: None,

            vnic_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VnicAttachmentLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set nic_index
    pub fn set_nic_index(mut self, value: Option<i64>) -> Self {
        self.nic_index = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set vlan_id
    pub fn set_vlan_id(mut self, value: Option<String>) -> Self {
        self.vlan_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vlan_tag
    pub fn set_vlan_tag(mut self, value: Option<i64>) -> Self {
        self.vlan_tag = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
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

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set vlan_id (unwraps Option)
    pub fn with_vlan_id(mut self, value: impl Into<String>) -> Self {
        self.vlan_id = Some(value.into());
        self
    }

    /// Set vlan_tag (unwraps Option)
    pub fn with_vlan_tag(mut self, value: i64) -> Self {
        self.vlan_tag = Some(value);
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }
}
