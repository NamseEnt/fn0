use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A connection between a DRG and CPE. This connection consists of multiple IPSec tunnels. Creating this connection is one of the steps required when setting up a Site-to-Site VPN. <p> *Important:**  Each tunnel in an IPSec connection can use either static routing or BGP dynamic routing (see the {@link IPSecConnectionTunnel} object's {@code routing} attribute). Originally only static routing was supported and every IPSec connection was required to have at least one static route configured. To maintain backward compatibility in the API when support for BPG dynamic routing was introduced, the API accepts an empty list of static routes if you configure both of the IPSec tunnels to use BGP dynamic routing. If you switch a tunnel's routing from {@code BGP} to {@code STATIC}, you must first ensure that the IPSec connection is configured with at least one valid CIDR block static route. Oracle uses the IPSec connection's static routes when routing a tunnel's traffic *only* if that tunnel's {@code routing} attribute = {@code STATIC}. Otherwise the static routes are ignored. <p> For more information about the workflow for setting up an IPSec connection, see [Site-to-Site VPN Overview](https://docs.oracle.com/iaas/Content/Network/Tasks/overviewIPsec.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPSecConnection {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPSec connection.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Cpe} object.
    pub cpe_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// The IPSec connection's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The IPSec connection's current state.
    pub lifecycle_state: IPSecConnectionLifecycleState,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Your identifier for your CPE device. Can be either an IP address or a hostname (specifically, the fully qualified domain name (FQDN)). The type of identifier here must correspond to the value for {@code cpeLocalIdentifierType}. <p> If you don't provide a value when creating the IPSec connection, the {@code ipAddress} attribute for the {@link Cpe} object specified by {@code cpeId} is used as the {@code cpeLocalIdentifier}. <p> For information about why you'd provide this value, see [If Your CPE Is Behind a NAT Device](https://docs.oracle.com/iaas/Content/Network/Tasks/overviewIPsec.htm#nat). <p> Example IP address: {@code 10.0.3.3} <p> Example hostname: {@code cpe.example.com}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_local_identifier: Option<String>,

    /// The type of identifier for your CPE device. The value here must correspond to the value for {@code cpeLocalIdentifier}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_local_identifier_type: Option<IPSecConnectionCpeLocalIdentifierType>,

    /// Static routes to the CPE. The CIDR must not be a multicast address or class E address. <p> Used for routing a given IPSec tunnel's traffic only if the tunnel is using static routing. If you configure at least one tunnel to use static routing, then you must provide at least one valid static route. If you configure both tunnels to use BGP dynamic routing, you can provide an empty list for the static routes. <p> The CIDR can be either IPv4 or IPv6. IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 10.0.1.0/24} <p> Example: {@code 2001:db8::/32}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_routes: Option<Vec<String>>,

    /// The date and time the IPSec connection was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The transport type used for the IPSec connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_type: Option<IPSecConnectionTransportType>,
}

/// Required fields for IPSecConnection
pub struct IPSecConnectionRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPSec connection.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Cpe} object.
    pub cpe_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// The IPSec connection's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The IPSec connection's current state.
    pub lifecycle_state: IPSecConnectionLifecycleState,
}

impl IPSecConnection {
    /// Create a new IPSecConnection with required fields
    pub fn new(required: IPSecConnectionRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            cpe_id: required.cpe_id,

            drg_id: required.drg_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            cpe_local_identifier: None,

            cpe_local_identifier_type: None,

            static_routes: None,

            time_created: None,

            transport_type: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set cpe_id
    pub fn set_cpe_id(mut self, value: String) -> Self {
        self.cpe_id = value;
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

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: IPSecConnectionLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set cpe_local_identifier
    pub fn set_cpe_local_identifier(mut self, value: Option<String>) -> Self {
        self.cpe_local_identifier = value;
        self
    }

    /// Set cpe_local_identifier_type
    pub fn set_cpe_local_identifier_type(
        mut self,
        value: Option<IPSecConnectionCpeLocalIdentifierType>,
    ) -> Self {
        self.cpe_local_identifier_type = value;
        self
    }

    /// Set static_routes
    pub fn set_static_routes(mut self, value: Option<Vec<String>>) -> Self {
        self.static_routes = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set transport_type
    pub fn set_transport_type(mut self, value: Option<IPSecConnectionTransportType>) -> Self {
        self.transport_type = value;
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

    /// Set cpe_local_identifier (unwraps Option)
    pub fn with_cpe_local_identifier(mut self, value: impl Into<String>) -> Self {
        self.cpe_local_identifier = Some(value.into());
        self
    }

    /// Set cpe_local_identifier_type (unwraps Option)
    pub fn with_cpe_local_identifier_type(
        mut self,
        value: IPSecConnectionCpeLocalIdentifierType,
    ) -> Self {
        self.cpe_local_identifier_type = Some(value);
        self
    }

    /// Set static_routes (unwraps Option)
    pub fn with_static_routes(mut self, value: Vec<String>) -> Self {
        self.static_routes = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set transport_type (unwraps Option)
    pub fn with_transport_type(mut self, value: IPSecConnectionTransportType) -> Self {
        self.transport_type = Some(value);
        self
    }
}
