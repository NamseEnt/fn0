use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeBackupPolicyAssetAssignmentRequest {
    /// The OCID of an asset (e.g. a volume).
    pub asset_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}


/// Required fields for GetVolumeBackupPolicyAssetAssignmentRequest
pub struct GetVolumeBackupPolicyAssetAssignmentRequestRequired {
    /// The OCID of an asset (e.g. a volume).
    pub asset_id: String,
}

impl GetVolumeBackupPolicyAssetAssignmentRequest {
    /// Create a new GetVolumeBackupPolicyAssetAssignmentRequest with required fields
    pub fn new(required: GetVolumeBackupPolicyAssetAssignmentRequestRequired) -> Self {
        Self {
            asset_id: required.asset_id,

            limit: None,

            page: None,
}
    }

    /// Set asset_id
    pub fn set_asset_id(mut self, value: String) -> Self {
        self.asset_id = value;
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
}


