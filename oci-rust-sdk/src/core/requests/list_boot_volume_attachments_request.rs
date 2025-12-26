use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBootVolumeAttachmentsRequest {
    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The OCID of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// The OCID of the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,
}


/// Required fields for ListBootVolumeAttachmentsRequest
pub struct ListBootVolumeAttachmentsRequestRequired {
    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListBootVolumeAttachmentsRequest {
    /// Create a new ListBootVolumeAttachmentsRequest with required fields
    pub fn new(required: ListBootVolumeAttachmentsRequestRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            limit: None,

            page: None,

            instance_id: None,

            boot_volume_id: None,
}
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
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

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: Option<String>) -> Self {
        self.boot_volume_id = value;
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

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set boot_volume_id (unwraps Option)
    pub fn with_boot_volume_id(mut self, value: impl Into<String>) -> Self {
        self.boot_volume_id = Some(value.into());
        self
    }
}


