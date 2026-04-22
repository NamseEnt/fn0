use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about an erratum associated with a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceErratumSummary {
    /// The identifier of the erratum.
    pub name: String,

    /// The advisory type of the erratum.
    pub advisory_type: ClassificationTypes,

    /// The list of packages affected by this erratum.
    pub packages: Vec<PackageNameSummary>,

    /// The date and time the package was issued by a providing erratum (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_issued: Option<DateTime<Utc>>,

    /// A summary description of the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,

    /// The list of CVEs applicable to this erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_cves: Option<Vec<String>>,
}

/// Required fields for ManagedInstanceErratumSummary
pub struct ManagedInstanceErratumSummaryRequired {
    /// The identifier of the erratum.
    pub name: String,

    /// The advisory type of the erratum.
    pub advisory_type: ClassificationTypes,

    /// The list of packages affected by this erratum.
    pub packages: Vec<PackageNameSummary>,
}

impl ManagedInstanceErratumSummary {
    /// Create a new ManagedInstanceErratumSummary with required fields
    pub fn new(required: ManagedInstanceErratumSummaryRequired) -> Self {
        Self {
            name: required.name,

            advisory_type: required.advisory_type,

            packages: required.packages,

            time_issued: None,

            synopsis: None,

            related_cves: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set advisory_type
    pub fn set_advisory_type(mut self, value: ClassificationTypes) -> Self {
        self.advisory_type = value;
        self
    }

    /// Set time_issued
    pub fn set_time_issued(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_issued = value;
        self
    }

    /// Set synopsis
    pub fn set_synopsis(mut self, value: Option<String>) -> Self {
        self.synopsis = value;
        self
    }

    /// Set related_cves
    pub fn set_related_cves(mut self, value: Option<Vec<String>>) -> Self {
        self.related_cves = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Vec<PackageNameSummary>) -> Self {
        self.packages = value;
        self
    }

    /// Set time_issued (unwraps Option)
    pub fn with_time_issued(mut self, value: DateTime<Utc>) -> Self {
        self.time_issued = Some(value);
        self
    }

    /// Set synopsis (unwraps Option)
    pub fn with_synopsis(mut self, value: impl Into<String>) -> Self {
        self.synopsis = Some(value.into());
        self
    }

    /// Set related_cves (unwraps Option)
    pub fn with_related_cves(mut self, value: Vec<String>) -> Self {
        self.related_cves = Some(value);
        self
    }
}
