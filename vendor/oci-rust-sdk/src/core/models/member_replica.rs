use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// OCIDs for the volume replicas in this volume group replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberReplica {
    /// The volume replica ID.
    pub volume_replica_id: String,

    /// Membership state of the volume replica in relation to the volume group replica.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership_state: Option<MemberReplicaMembershipState>,
}

/// Required fields for MemberReplica
pub struct MemberReplicaRequired {
    /// The volume replica ID.
    pub volume_replica_id: String,
}

impl MemberReplica {
    /// Create a new MemberReplica with required fields
    pub fn new(required: MemberReplicaRequired) -> Self {
        Self {
            volume_replica_id: required.volume_replica_id,

            membership_state: None,
        }
    }

    /// Set volume_replica_id
    pub fn set_volume_replica_id(mut self, value: String) -> Self {
        self.volume_replica_id = value;
        self
    }

    /// Set membership_state
    pub fn set_membership_state(mut self, value: Option<MemberReplicaMembershipState>) -> Self {
        self.membership_state = value;
        self
    }

    /// Set membership_state (unwraps Option)
    pub fn with_membership_state(mut self, value: MemberReplicaMembershipState) -> Self {
        self.membership_state = Some(value);
        self
    }
}
