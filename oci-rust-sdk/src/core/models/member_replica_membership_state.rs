use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberReplicaMembershipState {
    #[serde(rename = "ADD_PENDING")]
    AddPending,

    #[serde(rename = "STABLE")]
    Stable,

    #[serde(rename = "REMOVE_PENDING")]
    RemovePending,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
