use serde::{Deserialize, Serialize};

/// Details for attaching a VNIC to an instance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationAttachVnicDetails {
    /// Details for creating a new VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_vnic_details: Option<super::InstanceConfigurationCreateVnicDetails>,

    /// A user-friendly name. Does not have to be unique, and it's changeable.
    /// Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Which physical network interface card (NIC) the VNIC will use. Defaults to 0.
    /// Certain bare metal instance shapes have two active physical NICs (0 and 1). If
    /// you add a secondary VNIC to one of these instances, you can specify which NIC
    /// the VNIC will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nic_index: Option<i32>,
}

impl InstanceConfigurationAttachVnicDetails {
    pub fn new() -> Self {
        Self {
            create_vnic_details: None,
            display_name: None,
            nic_index: None,
        }
    }

    pub fn with_create_vnic_details(
        mut self,
        details: super::InstanceConfigurationCreateVnicDetails,
    ) -> Self {
        self.create_vnic_details = Some(details);
        self
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_nic_index(mut self, index: i32) -> Self {
        self.nic_index = Some(index);
        self
    }

    /// Set create_vnic_details
    pub fn set_create_vnic_details(mut self, create_vnic_details: Option<super::InstanceConfigurationCreateVnicDetails>) -> Self {
        self.create_vnic_details = create_vnic_details;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set nic_index
    pub fn set_nic_index(mut self, nic_index: Option<i32>) -> Self {
        self.nic_index = nic_index;
        self
    }
}

impl Default for InstanceConfigurationAttachVnicDetails {
    fn default() -> Self {
        Self::new()
    }
}
