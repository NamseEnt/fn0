use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create an entitlement using the Customer Support Identifier (CSI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntitlementDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy containing the entitlement.
    pub compartment_id: String,

    /// The Customer Support Identifier (CSI) which unlocks the software sources. The CSI is is a unique key given to a customer and it uniquely identifies the entitlement.
    pub csi: String,
}

/// Required fields for CreateEntitlementDetails
pub struct CreateEntitlementDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy containing the entitlement.
    pub compartment_id: String,

    /// The Customer Support Identifier (CSI) which unlocks the software sources. The CSI is is a unique key given to a customer and it uniquely identifies the entitlement.
    pub csi: String,
}

impl CreateEntitlementDetails {
    /// Create a new CreateEntitlementDetails with required fields
    pub fn new(required: CreateEntitlementDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            csi: required.csi,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set csi
    pub fn set_csi(mut self, value: String) -> Self {
        self.csi = value;
        self
    }
}
