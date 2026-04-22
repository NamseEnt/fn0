use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines an erratum..
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erratum {
    /// Advisory name.
    pub name: String,

    /// Summary description of the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,

    /// The date and time the erratum was issued (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_issued: Option<DateTime<Utc>>,

    /// Details describing the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The date and time the erratum was updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// Type of the erratum. This property is deprecated and it will be removed in a future API release. Please refer to the advisoryType property instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_type: Option<ClassificationTypes>,

    /// The advisory type of the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_type: Option<AdvisoryTypes>,

    /// Information specifying from where the erratum was release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// Information describing how the erratum can be resolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,

    /// Information describing how to find more information about. the erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<String>,

    /// List of CVEs applicable to this erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_cves: Option<Vec<String>>,

    /// List of repository identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<String>>,

    /// List of packages affected by this erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<SoftwarePackageSummary>>,

    /// List of affected OS families.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_families: Option<Vec<OsFamily>>,

    /// The severity for a security advisory, otherwise, null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_severity: Option<AdvisorySeverity>,
}

/// Required fields for Erratum
pub struct ErratumRequired {
    /// Advisory name.
    pub name: String,
}

impl Erratum {
    /// Create a new Erratum with required fields
    pub fn new(required: ErratumRequired) -> Self {
        Self {
            name: required.name,

            synopsis: None,

            time_issued: None,

            description: None,

            time_updated: None,

            classification_type: None,

            advisory_type: None,

            from: None,

            solution: None,

            references: None,

            related_cves: None,

            repositories: None,

            packages: None,

            os_families: None,

            advisory_severity: None,
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

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
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

    /// Set advisory_type
    pub fn set_advisory_type(mut self, value: Option<AdvisoryTypes>) -> Self {
        self.advisory_type = value;
        self
    }

    /// Set from
    pub fn set_from(mut self, value: Option<String>) -> Self {
        self.from = value;
        self
    }

    /// Set solution
    pub fn set_solution(mut self, value: Option<String>) -> Self {
        self.solution = value;
        self
    }

    /// Set references
    pub fn set_references(mut self, value: Option<String>) -> Self {
        self.references = value;
        self
    }

    /// Set related_cves
    pub fn set_related_cves(mut self, value: Option<Vec<String>>) -> Self {
        self.related_cves = value;
        self
    }

    /// Set repositories
    pub fn set_repositories(mut self, value: Option<Vec<String>>) -> Self {
        self.repositories = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Option<Vec<SoftwarePackageSummary>>) -> Self {
        self.packages = value;
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

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
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

    /// Set advisory_type (unwraps Option)
    pub fn with_advisory_type(mut self, value: AdvisoryTypes) -> Self {
        self.advisory_type = Some(value);
        self
    }

    /// Set from (unwraps Option)
    pub fn with_from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    /// Set solution (unwraps Option)
    pub fn with_solution(mut self, value: impl Into<String>) -> Self {
        self.solution = Some(value.into());
        self
    }

    /// Set references (unwraps Option)
    pub fn with_references(mut self, value: impl Into<String>) -> Self {
        self.references = Some(value.into());
        self
    }

    /// Set related_cves (unwraps Option)
    pub fn with_related_cves(mut self, value: Vec<String>) -> Self {
        self.related_cves = Some(value);
        self
    }

    /// Set repositories (unwraps Option)
    pub fn with_repositories(mut self, value: Vec<String>) -> Self {
        self.repositories = Some(value);
        self
    }

    /// Set packages (unwraps Option)
    pub fn with_packages(mut self, value: Vec<SoftwarePackageSummary>) -> Self {
        self.packages = Some(value);
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
}
