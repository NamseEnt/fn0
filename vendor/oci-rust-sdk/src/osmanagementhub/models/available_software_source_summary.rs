use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about an available software source. An available software source can be added to a managed instance. After a software source is added, packages from that software source can be installed on that managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableSoftwareSourceSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the software source.
    pub compartment_id: String,

    /// User-friendly name for the software source.
    pub display_name: String,
}

/// Required fields for AvailableSoftwareSourceSummary
pub struct AvailableSoftwareSourceSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the software source.
    pub compartment_id: String,

    /// User-friendly name for the software source.
    pub display_name: String,
}

impl AvailableSoftwareSourceSummary {
    /// Create a new AvailableSoftwareSourceSummary with required fields
    pub fn new(required: AvailableSoftwareSourceSummaryRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }
}
