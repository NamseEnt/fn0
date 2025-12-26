use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A remote peering connection (RPC) is an object on a DRG that lets the VCN that is attached to the DRG peer with a VCN in a different region. *Peering* means that the two VCNs can communicate using private IP addresses, but without the traffic traversing the internet or routing through your on-premises network. For more information, see [VCN Peering](https://docs.oracle.com/iaas/Content/Network/Tasks/VCNpeering.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotePeeringConnection {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the RPC.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG that this RPC belongs to.
    pub drg_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the RPC.
    pub id: String,

    /// Whether the VCN at the other end of the peering is in a different tenancy. <p> Example: {@code false}
    pub is_cross_tenancy_peering: bool,

    /// The RPC's current lifecycle state.
    pub lifecycle_state: RemotePeeringConnectionLifecycleState,

    /// Whether the RPC is peered with another RPC. {@code NEW} means the RPC has not yet been peered. {@code PENDING} means the peering is being established. {@code REVOKED} means the RPC at the other end of the peering has been deleted.
    pub peering_status: RemotePeeringConnectionPeeringStatus,

    /// The date and time the RPC was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// If this RPC is peered, this value is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the other RPC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,

    /// If this RPC is peered, this value is the region that contains the other RPC. <p> Example: {@code us-ashburn-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_region_name: Option<String>,

    /// If this RPC is peered, this value is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the other RPC's tenancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_tenancy_id: Option<String>,
}

/// Required fields for RemotePeeringConnection
pub struct RemotePeeringConnectionRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the RPC.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG that this RPC belongs to.
    pub drg_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the RPC.
    pub id: String,

    /// Whether the VCN at the other end of the peering is in a different tenancy. <p> Example: {@code false}
    pub is_cross_tenancy_peering: bool,

    /// The RPC's current lifecycle state.
    pub lifecycle_state: RemotePeeringConnectionLifecycleState,

    /// Whether the RPC is peered with another RPC. {@code NEW} means the RPC has not yet been peered. {@code PENDING} means the peering is being established. {@code REVOKED} means the RPC at the other end of the peering has been deleted.
    pub peering_status: RemotePeeringConnectionPeeringStatus,

    /// The date and time the RPC was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl RemotePeeringConnection {
    /// Create a new RemotePeeringConnection with required fields
    pub fn new(required: RemotePeeringConnectionRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            drg_id: required.drg_id,

            id: required.id,

            is_cross_tenancy_peering: required.is_cross_tenancy_peering,

            lifecycle_state: required.lifecycle_state,

            peering_status: required.peering_status,

            time_created: required.time_created,

            defined_tags: None,

            freeform_tags: None,

            peer_id: None,

            peer_region_name: None,

            peer_tenancy_id: None,
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
    pub fn set_display_name(mut self, value: String) -> Self {
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

    /// Set is_cross_tenancy_peering
    pub fn set_is_cross_tenancy_peering(mut self, value: bool) -> Self {
        self.is_cross_tenancy_peering = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: RemotePeeringConnectionLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set peer_id
    pub fn set_peer_id(mut self, value: Option<String>) -> Self {
        self.peer_id = value;
        self
    }

    /// Set peer_region_name
    pub fn set_peer_region_name(mut self, value: Option<String>) -> Self {
        self.peer_region_name = value;
        self
    }

    /// Set peer_tenancy_id
    pub fn set_peer_tenancy_id(mut self, value: Option<String>) -> Self {
        self.peer_tenancy_id = value;
        self
    }

    /// Set peering_status
    pub fn set_peering_status(mut self, value: RemotePeeringConnectionPeeringStatus) -> Self {
        self.peering_status = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set peer_id (unwraps Option)
    pub fn with_peer_id(mut self, value: impl Into<String>) -> Self {
        self.peer_id = Some(value.into());
        self
    }

    /// Set peer_region_name (unwraps Option)
    pub fn with_peer_region_name(mut self, value: impl Into<String>) -> Self {
        self.peer_region_name = Some(value.into());
        self
    }

    /// Set peer_tenancy_id (unwraps Option)
    pub fn with_peer_tenancy_id(mut self, value: impl Into<String>) -> Self {
        self.peer_tenancy_id = Some(value.into());
        self
    }
}
