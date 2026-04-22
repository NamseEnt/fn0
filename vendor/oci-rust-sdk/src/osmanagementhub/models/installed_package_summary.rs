use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a software package installed on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPackageSummary {
    /// The date and time the package was installed, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    pub time_installed: DateTime<Utc>,

    pub package_classification: String,

    /// The date and time the package was issued by a providing erratum (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_issued: Option<DateTime<Utc>>,
}

/// Required fields for InstalledPackageSummary
pub struct InstalledPackageSummaryRequired {
    /// The date and time the package was installed, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    pub time_installed: DateTime<Utc>,

    pub package_classification: String,
}

impl InstalledPackageSummary {
    /// Create a new InstalledPackageSummary with required fields
    pub fn new(required: InstalledPackageSummaryRequired) -> Self {
        Self {
            time_installed: required.time_installed,

            package_classification: required.package_classification,

            time_issued: None,
        }
    }

    /// Set time_installed
    pub fn set_time_installed(mut self, value: DateTime<Utc>) -> Self {
        self.time_installed = value;
        self
    }

    /// Set time_issued
    pub fn set_time_issued(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_issued = value;
        self
    }

    /// Set package_classification
    pub fn set_package_classification(mut self, value: String) -> Self {
        self.package_classification = value;
        self
    }

    /// Set time_issued (unwraps Option)
    pub fn with_time_issued(mut self, value: DateTime<Utc>) -> Self {
        self.time_issued = Some(value);
        self
    }
}
