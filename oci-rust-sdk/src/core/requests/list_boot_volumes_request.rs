use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBootVolumesRequest {
    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The OCID of the volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_id: Option<String>,
}

impl ListBootVolumesRequest {
    /// Create a new ListBootVolumesRequest
    pub fn new() -> Self {
        Self {
            availability_domain: None,

            compartment_id: None,

            limit: None,

            page: None,

            volume_group_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set limit
    pub fn set_limit(mut self, value: Option<i64>) -> Self {
        self.limit = value;
        self
    }

    /// Set page
    pub fn set_page(mut self, value: Option<String>) -> Self {
        self.page = value;
        self
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: Option<String>) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set limit (unwraps Option)
    pub fn with_limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    /// Set page (unwraps Option)
    pub fn with_page(mut self, value: impl Into<String>) -> Self {
        self.page = Some(value.into());
        self
    }

    /// Set volume_group_id (unwraps Option)
    pub fn with_volume_group_id(mut self, value: impl Into<String>) -> Self {
        self.volume_group_id = Some(value.into());
        self
    }
}

impl Default for ListBootVolumesRequest {
    fn default() -> Self {
        Self::new()
    }
}

