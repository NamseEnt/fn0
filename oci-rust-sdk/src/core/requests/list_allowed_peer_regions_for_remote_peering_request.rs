use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAllowedPeerRegionsForRemotePeeringRequest {}

impl ListAllowedPeerRegionsForRemotePeeringRequest {
    /// Create a new ListAllowedPeerRegionsForRemotePeeringRequest
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ListAllowedPeerRegionsForRemotePeeringRequest {
    fn default() -> Self {
        Self::new()
    }
}
