use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about a VNIC that belongs to a network security group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSecurityGroupVnic {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the parent resource that the VNIC is attached to (for example, a Compute instance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// The date and time the VNIC was added to the network security group, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_associated: Option<DateTime<Utc>>,
}

/// Required fields for NetworkSecurityGroupVnic
pub struct NetworkSecurityGroupVnicRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub vnic_id: String,
}

impl NetworkSecurityGroupVnic {
    /// Create a new NetworkSecurityGroupVnic with required fields
    pub fn new(required: NetworkSecurityGroupVnicRequired) -> Self {
        Self {
            vnic_id: required.vnic_id,

            resource_id: None,

            time_associated: None,
        }
    }

    /// Set resource_id
    pub fn set_resource_id(mut self, value: Option<String>) -> Self {
        self.resource_id = value;
        self
    }

    /// Set time_associated
    pub fn set_time_associated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_associated = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: String) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set resource_id (unwraps Option)
    pub fn with_resource_id(mut self, value: impl Into<String>) -> Self {
        self.resource_id = Some(value.into());
        self
    }

    /// Set time_associated (unwraps Option)
    pub fn with_time_associated(mut self, value: DateTime<Utc>) -> Self {
        self.time_associated = Some(value);
        self
    }
}
