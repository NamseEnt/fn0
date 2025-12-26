use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeleteVirtualCircuitPublicPrefixesResponse {}

impl BulkDeleteVirtualCircuitPublicPrefixesResponse {
    /// Create a new BulkDeleteVirtualCircuitPublicPrefixesResponse
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BulkDeleteVirtualCircuitPublicPrefixesResponse {
    fn default() -> Self {
        Self::new()
    }
}
