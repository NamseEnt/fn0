use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Information to create a virtual network interface card (VNIC) which gives the containers on this container instance access to a virtual client network (VCN). <p> You use this object when creating the primary VNIC during container instance launch or when creating a secondary VNIC. This VNIC is created in the same compartment as the specified subnet on behalf of the customer. <p> The VNIC created by this call contains both the tags specified in this object as well as any tags specified in the parent container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerVnicDetails {
    /// The OCID of the subnet to create the VNIC in.
    pub subnet_id: String,

    /// A user-friendly name for the VNIC. Does not have to be unique. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The hostname for the VNIC's primary private IP. Used for DNS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// Whether the VNIC should be assigned a public IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public_ip_assigned: Option<bool>,

    /// Whether the source/destination check is disabled on the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// A list of the OCIDs of the network security groups (NSGs) to add the VNIC to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// A private IP address of your choice to assign to the VNIC. Must be an available IP address within the subnet's CIDR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// Simple key-value pair that is applied without any predefined name, type or scope. Exists for cross-compatibility only. Example: {@code {\"bar-key\": \"value\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateContainerVnicDetails
pub struct CreateContainerVnicDetailsRequired {
    /// The OCID of the subnet to create the VNIC in.
    pub subnet_id: String,
}

impl CreateContainerVnicDetails {
    /// Create a new CreateContainerVnicDetails with required fields
    pub fn new(required: CreateContainerVnicDetailsRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,

            display_name: None,

            hostname_label: None,

            is_public_ip_assigned: None,

            skip_source_dest_check: None,

            nsg_ids: None,

            private_ip: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set hostname_label
    pub fn set_hostname_label(mut self, value: Option<String>) -> Self {
        self.hostname_label = value;
        self
    }

    /// Set is_public_ip_assigned
    pub fn set_is_public_ip_assigned(mut self, value: Option<bool>) -> Self {
        self.is_public_ip_assigned = value;
        self
    }

    /// Set skip_source_dest_check
    pub fn set_skip_source_dest_check(mut self, value: Option<bool>) -> Self {
        self.skip_source_dest_check = value;
        self
    }

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set private_ip
    pub fn set_private_ip(mut self, value: Option<String>) -> Self {
        self.private_ip = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
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

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set hostname_label (unwraps Option)
    pub fn with_hostname_label(mut self, value: impl Into<String>) -> Self {
        self.hostname_label = Some(value.into());
        self
    }

    /// Set is_public_ip_assigned (unwraps Option)
    pub fn with_is_public_ip_assigned(mut self, value: bool) -> Self {
        self.is_public_ip_assigned = Some(value);
        self
    }

    /// Set skip_source_dest_check (unwraps Option)
    pub fn with_skip_source_dest_check(mut self, value: bool) -> Self {
        self.skip_source_dest_check = Some(value);
        self
    }

    /// Set nsg_ids (unwraps Option)
    pub fn with_nsg_ids(mut self, value: Vec<String>) -> Self {
        self.nsg_ids = Some(value);
        self
    }

    /// Set private_ip (unwraps Option)
    pub fn with_private_ip(mut self, value: impl Into<String>) -> Self {
        self.private_ip = Some(value.into());
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
