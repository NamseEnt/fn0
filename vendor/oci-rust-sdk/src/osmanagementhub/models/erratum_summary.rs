use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for an erratum. An erratum is an important software change which can include security advisories, bug fixes, or enhancements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErratumSummary {
    /// Advisory name.
    pub name: String,

    /// Summary description of the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,

    /// The date and time the erratum was issued (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_issued: Option<DateTime<Utc>>,

    /// The date and time the erratum was updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// Type of the erratum. This property is deprecated and it will be removed in a future API release. Please refer to the advisoryType property instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_type: Option<ClassificationTypes>,

    /// List of CVEs applicable to this erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_cves: Option<Vec<String>>,

    /// List of affected OS families.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_families: Option<Vec<OsFamily>>,

    /// The severity advisory. Only valid for security type advisories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_severity: Option<AdvisorySeverity>,

    /// The advisory type of the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_type: Option<AdvisoryTypes>,
}

/// Required fields for ErratumSummary
pub struct ErratumSummaryRequired {
    /// Advisory name.
    pub name: String,
}

impl ErratumSummary {
    /// Create a new ErratumSummary with required fields
    pub fn new(required: ErratumSummaryRequired) -> Self {
        Self {
            name: required.name,

            synopsis: None,

            time_issued: None,

            time_updated: None,

            classification_type: None,

            related_cves: None,

            os_families: None,

            advisory_severity: None,

            advisory_type: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set synopsis
    pub fn set_synopsis(mut self, value: Option<String>) -> Self {
        self.synopsis = value;
        self
    }

    /// Set time_issued
    pub fn set_time_issued(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_issued = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set classification_type
    pub fn set_classification_type(mut self, value: Option<ClassificationTypes>) -> Self {
        self.classification_type = value;
        self
    }

    /// Set related_cves
    pub fn set_related_cves(mut self, value: Option<Vec<String>>) -> Self {
        self.related_cves = value;
        self
    }

    /// Set os_families
    pub fn set_os_families(mut self, value: Option<Vec<OsFamily>>) -> Self {
        self.os_families = value;
        self
    }

    /// Set advisory_severity
    pub fn set_advisory_severity(mut self, value: Option<AdvisorySeverity>) -> Self {
        self.advisory_severity = value;
        self
    }

    /// Set advisory_type
    pub fn set_advisory_type(mut self, value: Option<AdvisoryTypes>) -> Self {
        self.advisory_type = value;
        self
    }

    /// Set synopsis (unwraps Option)
    pub fn with_synopsis(mut self, value: impl Into<String>) -> Self {
        self.synopsis = Some(value.into());
        self
    }

    /// Set time_issued (unwraps Option)
    pub fn with_time_issued(mut self, value: DateTime<Utc>) -> Self {
        self.time_issued = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set classification_type (unwraps Option)
    pub fn with_classification_type(mut self, value: ClassificationTypes) -> Self {
        self.classification_type = Some(value);
        self
    }

    /// Set related_cves (unwraps Option)
    pub fn with_related_cves(mut self, value: Vec<String>) -> Self {
        self.related_cves = Some(value);
        self
    }

    /// Set os_families (unwraps Option)
    pub fn with_os_families(mut self, value: Vec<OsFamily>) -> Self {
        self.os_families = Some(value);
        self
    }

    /// Set advisory_severity (unwraps Option)
    pub fn with_advisory_severity(mut self, value: AdvisorySeverity) -> Self {
        self.advisory_severity = Some(value);
        self
    }

    /// Set advisory_type (unwraps Option)
    pub fn with_advisory_type(mut self, value: AdvisoryTypes) -> Self {
        self.advisory_type = Some(value);
        self
    }
}
