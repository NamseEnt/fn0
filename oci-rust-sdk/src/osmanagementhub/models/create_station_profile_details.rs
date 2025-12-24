use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create the management station profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStationProfileDetails {
    pub profile_type: String,

    /// The vendor of the operating system for the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<VendorName>,

    /// The operating system family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The architecture type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,
}

/// Required fields for CreateStationProfileDetails
pub struct CreateStationProfileDetailsRequired {
    pub profile_type: String,
}

impl CreateStationProfileDetails {
    /// Create a new CreateStationProfileDetails with required fields
    pub fn new(required: CreateStationProfileDetailsRequired) -> Self {
        Self {
            profile_type: required.profile_type,

            vendor_name: None,

            os_family: None,

            arch_type: None,
        }
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: Option<VendorName>) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: Option<OsFamily>) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: Option<ArchType>) -> Self {
        self.arch_type = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }

    /// Set vendor_name (unwraps Option)
    pub fn with_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = Some(value);
        self
    }

    /// Set os_family (unwraps Option)
    pub fn with_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = Some(value);
        self
    }

    /// Set arch_type (unwraps Option)
    pub fn with_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = Some(value);
        self
    }
}
