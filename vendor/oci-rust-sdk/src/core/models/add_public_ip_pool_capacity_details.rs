use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The information used to add capacity to an IP pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPublicIpPoolCapacityDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoip_range_id: String,

    /// The CIDR block to add to the public IP pool. It could be all of the CIDR block identified in {@code byoipRangeId}, or a subrange. Example: {@code 10.0.1.0/24}
    pub cidr_block: String,
}

/// Required fields for AddPublicIpPoolCapacityDetails
pub struct AddPublicIpPoolCapacityDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoip_range_id: String,

    /// The CIDR block to add to the public IP pool. It could be all of the CIDR block identified in {@code byoipRangeId}, or a subrange. Example: {@code 10.0.1.0/24}
    pub cidr_block: String,
}

impl AddPublicIpPoolCapacityDetails {
    /// Create a new AddPublicIpPoolCapacityDetails with required fields
    pub fn new(required: AddPublicIpPoolCapacityDetailsRequired) -> Self {
        Self {
            byoip_range_id: required.byoip_range_id,

            cidr_block: required.cidr_block,
        }
    }

    /// Set byoip_range_id
    pub fn set_byoip_range_id(mut self, value: String) -> Self {
        self.byoip_range_id = value;
        self
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
        self
    }
}
