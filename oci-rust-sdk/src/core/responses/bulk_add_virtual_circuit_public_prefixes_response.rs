use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkAddVirtualCircuitPublicPrefixesResponse {}

impl BulkAddVirtualCircuitPublicPrefixesResponse {
    /// Create a new BulkAddVirtualCircuitPublicPrefixesResponse
    pub fn new() -> Self {
        Self {        }
    }
}

impl Default for BulkAddVirtualCircuitPublicPrefixesResponse {
    fn default() -> Self {
        Self::new()
    }
}

