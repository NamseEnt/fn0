use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a Windows standalone registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWindowsStandAloneProfileDetails {
    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,
}

/// Required fields for CreateWindowsStandAloneProfileDetails
pub struct CreateWindowsStandAloneProfileDetailsRequired {
    /// The vendor of the operating system for the instance.
    pub vendor_name: VendorName,

    /// The operating system family.
    pub os_family: OsFamily,

    /// The architecture type.
    pub arch_type: ArchType,

    pub profile_type: String,
}

impl CreateWindowsStandAloneProfileDetails {
    /// Create a new CreateWindowsStandAloneProfileDetails with required fields
    pub fn new(required: CreateWindowsStandAloneProfileDetailsRequired) -> Self {
        Self {
            vendor_name: required.vendor_name,

            os_family: required.os_family,

            arch_type: required.arch_type,

            profile_type: required.profile_type,
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

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
