use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a software source mirror.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// Current state of the software source mirror.
    pub state: MirrorState,

    /// A decimal number representing the percentage of the software source that has been synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percentage: i64,

    /// Time that the software source was last synced (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_last_synced: DateTime<Utc>,

    /// The current log from the management station plugin.
    pub log: String,

    /// The number of packages within the mirrored software source. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub package_count: i64,

    /// The size the mirrored software source in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// Display name of the mirror.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Type of software source.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<MirrorType>,

    /// The OS family of the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// The architecture type supported by the software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch_type: Option<ArchType>,
}

/// Required fields for MirrorSummary
pub struct MirrorSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source.
    pub id: String,

    /// Current state of the software source mirror.
    pub state: MirrorState,

    /// A decimal number representing the percentage of the software source that has been synced. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percentage: i64,

    /// Time that the software source was last synced (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_last_synced: DateTime<Utc>,

    /// The current log from the management station plugin.
    pub log: String,

    /// The number of packages within the mirrored software source. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub package_count: i64,

    /// The size the mirrored software source in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,
}

impl MirrorSummary {
    /// Create a new MirrorSummary with required fields
    pub fn new(required: MirrorSummaryRequired) -> Self {
        Self {
            id: required.id,

            state: required.state,

            percentage: required.percentage,

            time_last_synced: required.time_last_synced,

            log: required.log,

            package_count: required.package_count,

            size: required.size,

            display_name: None,

            r#type: None,

            os_family: None,

            arch_type: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<MirrorType>) -> Self {
        self.r#type = value;
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

    /// Set state
    pub fn set_state(mut self, value: MirrorState) -> Self {
        self.state = value;
        self
    }

    /// Set percentage
    pub fn set_percentage(mut self, value: i64) -> Self {
        self.percentage = value;
        self
    }

    /// Set time_last_synced
    pub fn set_time_last_synced(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_synced = value;
        self
    }

    /// Set log
    pub fn set_log(mut self, value: String) -> Self {
        self.log = value;
        self
    }

    /// Set package_count
    pub fn set_package_count(mut self, value: i64) -> Self {
        self.package_count = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: i64) -> Self {
        self.size = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: MirrorType) -> Self {
        self.r#type = Some(value);
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
