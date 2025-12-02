use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Details for creating a VNIC (Virtual Network Interface Card)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVnicDetails {
    /// The OCID of the subnet to create the VNIC in (required)
    pub subnet_id: String,

    /// Whether to assign a public IP address to the VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_public_ip: Option<bool>,

    /// Whether to assign a private DNS record to the VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_private_dns_record: Option<bool>,

    /// A user-friendly name for the VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The hostname for the VNIC's primary private IP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// A private IP address to assign to this VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// A list of the OCIDs of the network security groups to add the VNIC to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// Whether the source/destination check is disabled on the VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// Defined tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

impl CreateVnicDetails {
    /// Create new VNIC details with required subnet_id
    pub fn new(subnet_id: impl Into<String>) -> Self {
        Self {
            subnet_id: subnet_id.into(),
            assign_public_ip: None,
            assign_private_dns_record: None,
            display_name: None,
            hostname_label: None,
            private_ip: None,
            nsg_ids: None,
            skip_source_dest_check: None,
            defined_tags: None,
            freeform_tags: None,
        }
    }

    /// Set whether to assign a public IP
    pub fn with_assign_public_ip(mut self, assign: bool) -> Self {
        self.assign_public_ip = Some(assign);
        self
    }

    /// Set the display name
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set the hostname label
    pub fn with_hostname_label(mut self, label: impl Into<String>) -> Self {
        self.hostname_label = Some(label.into());
        self
    }

    /// Set a specific private IP address
    pub fn with_private_ip(mut self, ip: impl Into<String>) -> Self {
        self.private_ip = Some(ip.into());
        self
    }

    /// Set network security groups
    pub fn with_nsg_ids(mut self, nsg_ids: Vec<String>) -> Self {
        self.nsg_ids = Some(nsg_ids);
        self
    }

    /// Set whether to skip source/destination check
    pub fn with_skip_source_dest_check(mut self, skip: bool) -> Self {
        self.skip_source_dest_check = Some(skip);
        self
    }
}
