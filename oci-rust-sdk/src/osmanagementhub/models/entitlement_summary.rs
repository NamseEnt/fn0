use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for an entitlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitlementSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy containing the entitlement.
    pub compartment_id: String,

    /// The Customer Support Identifier (CSI) which unlocks the software sources. The CSI is is a unique key given to a customer and it uniquely identifies the entitlement.
    pub csi: String,

    /// The vendor for the entitlement.
    pub vendor_name: String,
}

/// Required fields for EntitlementSummary
pub struct EntitlementSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenancy containing the entitlement.
    pub compartment_id: String,

    /// The Customer Support Identifier (CSI) which unlocks the software sources. The CSI is is a unique key given to a customer and it uniquely identifies the entitlement.
    pub csi: String,

    /// The vendor for the entitlement.
    pub vendor_name: String,
}

impl EntitlementSummary {
    /// Create a new EntitlementSummary with required fields
    pub fn new(required: EntitlementSummaryRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            csi: required.csi,

            vendor_name: required.vendor_name,
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

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: String) -> Self {
        self.vendor_name = value;
        self
    }
}
