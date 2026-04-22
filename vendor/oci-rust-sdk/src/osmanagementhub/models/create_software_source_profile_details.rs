use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create the software source registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSoftwareSourceProfileDetails {
    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,

    /// The list of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that the registration profile will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_ids: Option<Vec<String>>,
}

/// Required fields for CreateSoftwareSourceProfileDetails
pub struct CreateSoftwareSourceProfileDetailsRequired {
    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,
}

impl CreateSoftwareSourceProfileDetails {
    /// Create a new CreateSoftwareSourceProfileDetails with required fields
    pub fn new(required: CreateSoftwareSourceProfileDetailsRequired) -> Self {
        Self {
            vendor_name: required.vendor_name,

            os_family: required.os_family,

            arch_type: required.arch_type,

            profile_type: required.profile_type,

            software_source_ids: None,
        }
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = value;
        self
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }

    /// Set software_source_ids (unwraps Option)
    pub fn with_software_source_ids(mut self, value: Vec<String>) -> Self {
        self.software_source_ids = Some(value);
        self
    }
}
