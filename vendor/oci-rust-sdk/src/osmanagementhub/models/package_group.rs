use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Yum or DNF package group, category, or environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageGroup {
    /// Package group identifier.
    pub id: String,

    /// Package group name.
    pub name: String,

    /// The list of packages in the package group.
    pub packages: Vec<String>,

    /// The repository IDs of the package group's repositories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<String>>,

    /// Description of the package group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Indicates if this package group is visible to users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user_visible: Option<bool>,

    /// Indicates if this package group is the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// Indicates if this is a group, category, or environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_type: Option<PackageGroupGroupType>,

    /// Indicates the order to display category or environment. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_order: Option<i64>,
}

/// Required fields for PackageGroup
pub struct PackageGroupRequired {
    /// Package group identifier.
    pub id: String,

    /// Package group name.
    pub name: String,

    /// The list of packages in the package group.
    pub packages: Vec<String>,
}

impl PackageGroup {
    /// Create a new PackageGroup with required fields
    pub fn new(required: PackageGroupRequired) -> Self {
        Self {
            id: required.id,

            name: required.name,

            packages: required.packages,

            repositories: None,

            description: None,

            is_user_visible: None,

            is_default: None,

            group_type: None,

            display_order: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set repositories
    pub fn set_repositories(mut self, value: Option<Vec<String>>) -> Self {
        self.repositories = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set is_user_visible
    pub fn set_is_user_visible(mut self, value: Option<bool>) -> Self {
        self.is_user_visible = value;
        self
    }

    /// Set is_default
    pub fn set_is_default(mut self, value: Option<bool>) -> Self {
        self.is_default = value;
        self
    }

    /// Set group_type
    pub fn set_group_type(mut self, value: Option<PackageGroupGroupType>) -> Self {
        self.group_type = value;
        self
    }

    /// Set display_order
    pub fn set_display_order(mut self, value: Option<i64>) -> Self {
        self.display_order = value;
        self
    }

    /// Set packages
    pub fn set_packages(mut self, value: Vec<String>) -> Self {
        self.packages = value;
        self
    }

    /// Set repositories (unwraps Option)
    pub fn with_repositories(mut self, value: Vec<String>) -> Self {
        self.repositories = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set is_user_visible (unwraps Option)
    pub fn with_is_user_visible(mut self, value: bool) -> Self {
        self.is_user_visible = Some(value);
        self
    }

    /// Set is_default (unwraps Option)
    pub fn with_is_default(mut self, value: bool) -> Self {
        self.is_default = Some(value);
        self
    }

    /// Set group_type (unwraps Option)
    pub fn with_group_type(mut self, value: PackageGroupGroupType) -> Self {
        self.group_type = Some(value);
        self
    }

    /// Set display_order (unwraps Option)
    pub fn with_display_order(mut self, value: i64) -> Self {
        self.display_order = Some(value);
        self
    }
}
