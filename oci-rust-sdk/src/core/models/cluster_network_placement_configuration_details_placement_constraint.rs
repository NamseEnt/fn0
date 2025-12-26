use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterNetworkPlacementConfigurationDetailsPlacementConstraint {
    #[serde(rename = "SINGLE_TIER")]
    SingleTier,

    #[serde(rename = "SINGLE_BLOCK")]
    SingleBlock,

    #[serde(rename = "PACKED_DISTRIBUTION_MULTI_BLOCK")]
    PackedDistributionMultiBlock,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
