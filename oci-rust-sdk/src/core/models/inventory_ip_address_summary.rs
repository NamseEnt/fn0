use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the IP address and its corresponding VNIC ID, VNIC name, and DNS hostname.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryIpAddressSummary {
    /// The IP address assigned from a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,

    /// The name of the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_name: Option<String>,

    /// The DNS hostname of the resource assigned with the IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_host_name: Option<String>,
}

impl InventoryIpAddressSummary {
    /// Create a new InventoryIpAddressSummary
    pub fn new() -> Self {
        Self {
            ip_address: None,

            vnic_id: None,

            vnic_name: None,

            dns_host_name: None,
        }
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set vnic_name
    pub fn set_vnic_name(mut self, value: Option<String>) -> Self {
        self.vnic_name = value;
        self
    }

    /// Set dns_host_name
    pub fn set_dns_host_name(mut self, value: Option<String>) -> Self {
        self.dns_host_name = value;
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }

    /// Set vnic_name (unwraps Option)
    pub fn with_vnic_name(mut self, value: impl Into<String>) -> Self {
        self.vnic_name = Some(value.into());
        self
    }

    /// Set dns_host_name (unwraps Option)
    pub fn with_dns_host_name(mut self, value: impl Into<String>) -> Self {
        self.dns_host_name = Some(value.into());
        self
    }
}

impl Default for InventoryIpAddressSummary {
    fn default() -> Self {
        Self::new()
    }
}
