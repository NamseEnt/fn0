use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a software package available for installation on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatablePackageSummary {
    /// The type of update.
    pub update_type: ClassificationTypes,

    pub package_classification: String,

    /// The version of the package that is currently installed on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_version: Option<String>,

    /// List of errata applicable to this update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errata: Option<Vec<String>>,

    /// List of CVEs applicable to this erratum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_cves: Option<Vec<String>>,
}

/// Required fields for UpdatablePackageSummary
pub struct UpdatablePackageSummaryRequired {
    /// The type of update.
    pub update_type: ClassificationTypes,

    pub package_classification: String,
}

impl UpdatablePackageSummary {
    /// Create a new UpdatablePackageSummary with required fields
    pub fn new(required: UpdatablePackageSummaryRequired) -> Self {
        Self {
            update_type: required.update_type,

            package_classification: required.package_classification,

            installed_version: None,

            errata: None,

            related_cves: None,
        }
    }

    /// Set installed_version
    pub fn set_installed_version(mut self, value: Option<String>) -> Self {
        self.installed_version = value;
        self
    }

    /// Set update_type
    pub fn set_update_type(mut self, value: ClassificationTypes) -> Self {
        self.update_type = value;
        self
    }

    /// Set errata
    pub fn set_errata(mut self, value: Option<Vec<String>>) -> Self {
        self.errata = value;
        self
    }

    /// Set related_cves
    pub fn set_related_cves(mut self, value: Option<Vec<String>>) -> Self {
        self.related_cves = value;
        self
    }

    /// Set package_classification
    pub fn set_package_classification(mut self, value: String) -> Self {
        self.package_classification = value;
        self
    }

    /// Set installed_version (unwraps Option)
    pub fn with_installed_version(mut self, value: impl Into<String>) -> Self {
        self.installed_version = Some(value.into());
        self
    }

    /// Set errata (unwraps Option)
    pub fn with_errata(mut self, value: Vec<String>) -> Self {
        self.errata = Some(value);
        self
    }

    /// Set related_cves (unwraps Option)
    pub fn with_related_cves(mut self, value: Vec<String>) -> Self {
        self.related_cves = Some(value);
        self
    }
}
