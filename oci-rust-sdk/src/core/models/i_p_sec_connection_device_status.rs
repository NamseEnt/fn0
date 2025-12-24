use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Deprecated. For tunnel information, instead see {@link IPSecConnectionTunnel}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPSecConnectionDeviceStatus {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPSec connection.
    pub compartment_id: String,

    /// The IPSec connection's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The date and time the IPSec connection was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// Two {@link TunnelStatus} objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnels: Option<Vec<TunnelStatus>>,
}

/// Required fields for IPSecConnectionDeviceStatus
pub struct IPSecConnectionDeviceStatusRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPSec connection.
    pub compartment_id: String,

    /// The IPSec connection's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,
}

impl IPSecConnectionDeviceStatus {
    /// Create a new IPSecConnectionDeviceStatus with required fields
    pub fn new(required: IPSecConnectionDeviceStatusRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            time_created: None,

            tunnels: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set tunnels
    pub fn set_tunnels(mut self, value: Option<Vec<TunnelStatus>>) -> Self {
        self.tunnels = value;
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set tunnels (unwraps Option)
    pub fn with_tunnels(mut self, value: Vec<TunnelStatus>) -> Self {
        self.tunnels = Some(value);
        self
    }
}
