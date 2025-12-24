use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The object that defines a software source. A software source contains a collection of packages. For more information, see [Managing Software Sources](https://docs.oracle.com/iaas/osmh/doc/software-sources.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSource {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the software source.
    pub compartment_id: String,

    /// User-friendly name for the software source.
    pub display_name: String,

    /// The date and time the software source was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,

    /// Availability of the software source (for non-OCI environments).
    pub availability: Availability,

    /// Availability of the software source (for OCI environments).
    pub availability_at_oci: Availability,

    /// The repository ID for the software source.
    pub repo_id: String,

    /// The OS family of the software source.
    pub os_family: OsFamily,

    /// The architecture type supported by the software source.
    pub arch_type: ArchType,

    /// URL for the repository. For vendor software sources, this is the URL to the regional yum server. For custom software sources, this is 'custom/<repoId>'.
    pub url: String,

    pub software_source_type: String,

    /// User-specified description for the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The current state of the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<SoftwareSourceLifecycleState>,

    /// Number of packages the software source contains. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_count: Option<i64>,

    /// The yum repository checksum type used by this software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_type: Option<ChecksumType>,

    /// URI of the GPG key for this software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key_url: Option<String>,

    /// ID of the GPG key for this software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key_id: Option<String>,

    /// Fingerprint of the GPG key for this software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key_fingerprint: Option<String>,

    /// The size of the software source in bytes (B). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for SoftwareSource
pub struct SoftwareSourceRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the software source.
    pub compartment_id: String,

    /// User-friendly name for the software source.
    pub display_name: String,

    /// The date and time the software source was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,

    /// Availability of the software source (for non-OCI environments).
    pub availability: Availability,

    /// Availability of the software source (for OCI environments).
    pub availability_at_oci: Availability,

    /// The repository ID for the software source.
    pub repo_id: String,

    /// The OS family of the software source.
    pub os_family: OsFamily,

    /// The architecture type supported by the software source.
    pub arch_type: ArchType,

    /// URL for the repository. For vendor software sources, this is the URL to the regional yum server. For custom software sources, this is 'custom/<repoId>'.
    pub url: String,

    pub software_source_type: String,
}

impl SoftwareSource {
    /// Create a new SoftwareSource with required fields
    pub fn new(required: SoftwareSourceRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            display_name: required.display_name,

            time_created: required.time_created,

            availability: required.availability,

            availability_at_oci: required.availability_at_oci,

            repo_id: required.repo_id,

            os_family: required.os_family,

            arch_type: required.arch_type,

            url: required.url,

            software_source_type: required.software_source_type,

            description: None,

            lifecycle_state: None,

            package_count: None,

            checksum_type: None,

            gpg_key_url: None,

            gpg_key_id: None,

            gpg_key_fingerprint: None,

            size: None,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,
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

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set availability
    pub fn set_availability(mut self, value: Availability) -> Self {
        self.availability = value;
        self
    }

    /// Set availability_at_oci
    pub fn set_availability_at_oci(mut self, value: Availability) -> Self {
        self.availability_at_oci = value;
        self
    }

    /// Set repo_id
    pub fn set_repo_id(mut self, value: String) -> Self {
        self.repo_id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<SoftwareSourceLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set package_count
    pub fn set_package_count(mut self, value: Option<i64>) -> Self {
        self.package_count = value;
        self
    }

    /// Set url
    pub fn set_url(mut self, value: String) -> Self {
        self.url = value;
        self
    }

    /// Set checksum_type
    pub fn set_checksum_type(mut self, value: Option<ChecksumType>) -> Self {
        self.checksum_type = value;
        self
    }

    /// Set gpg_key_url
    pub fn set_gpg_key_url(mut self, value: Option<String>) -> Self {
        self.gpg_key_url = value;
        self
    }

    /// Set gpg_key_id
    pub fn set_gpg_key_id(mut self, value: Option<String>) -> Self {
        self.gpg_key_id = value;
        self
    }

    /// Set gpg_key_fingerprint
    pub fn set_gpg_key_fingerprint(mut self, value: Option<String>) -> Self {
        self.gpg_key_fingerprint = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: Option<i64>) -> Self {
        self.size = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: SoftwareSourceLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set package_count (unwraps Option)
    pub fn with_package_count(mut self, value: i64) -> Self {
        self.package_count = Some(value);
        self
    }

    /// Set checksum_type (unwraps Option)
    pub fn with_checksum_type(mut self, value: ChecksumType) -> Self {
        self.checksum_type = Some(value);
        self
    }

    /// Set gpg_key_url (unwraps Option)
    pub fn with_gpg_key_url(mut self, value: impl Into<String>) -> Self {
        self.gpg_key_url = Some(value.into());
        self
    }

    /// Set gpg_key_id (unwraps Option)
    pub fn with_gpg_key_id(mut self, value: impl Into<String>) -> Self {
        self.gpg_key_id = Some(value.into());
        self
    }

    /// Set gpg_key_fingerprint (unwraps Option)
    pub fn with_gpg_key_fingerprint(mut self, value: impl Into<String>) -> Self {
        self.gpg_key_fingerprint = Some(value.into());
        self
    }

    /// Set size (unwraps Option)
    pub fn with_size(mut self, value: i64) -> Self {
        self.size = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }
}
