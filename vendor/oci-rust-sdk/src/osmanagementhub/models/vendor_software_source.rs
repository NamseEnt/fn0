use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The object that defines a vendor software source. A software source is a collection of packages. For more information, see [Managing Software Sources](https://docs.oracle.com/iaas/osmh/doc/software-sources.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorSoftwareSource {
    /// Name of the vendor providing the software source.
    pub vendor_name: VendorName,

    pub software_source_type: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the vendor software source in the root compartment. This property applies only to replicated vendor software sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_software_source_id: Option<String>,

    /// Indicates whether the software source is required for the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory_for_autonomous_linux: Option<bool>,
}

/// Required fields for VendorSoftwareSource
pub struct VendorSoftwareSourceRequired {
    /// Name of the vendor providing the software source.
    pub vendor_name: VendorName,

    pub software_source_type: String,
}

impl VendorSoftwareSource {
    /// Create a new VendorSoftwareSource with required fields
    pub fn new(required: VendorSoftwareSourceRequired) -> Self {
        Self {
            vendor_name: required.vendor_name,

            software_source_type: required.software_source_type,

            origin_software_source_id: None,

            is_mandatory_for_autonomous_linux: None,
        }
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set origin_software_source_id
    pub fn set_origin_software_source_id(mut self, value: Option<String>) -> Self {
        self.origin_software_source_id = value;
        self
    }

    /// Set is_mandatory_for_autonomous_linux
    pub fn set_is_mandatory_for_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_mandatory_for_autonomous_linux = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set origin_software_source_id (unwraps Option)
    pub fn with_origin_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.origin_software_source_id = Some(value.into());
        self
    }

    /// Set is_mandatory_for_autonomous_linux (unwraps Option)
    pub fn with_is_mandatory_for_autonomous_linux(mut self, value: bool) -> Self {
        self.is_mandatory_for_autonomous_linux = Some(value);
        self
    }
}
