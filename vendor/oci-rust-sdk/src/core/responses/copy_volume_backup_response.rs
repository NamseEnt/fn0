use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyVolumeBackupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// Location of the resource.
    pub location: String,

    /// Location of the resource.
    pub content_location: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}

/// Required fields for CopyVolumeBackupResponse
pub struct CopyVolumeBackupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// Location of the resource.
    pub location: String,

    /// Location of the resource.
    pub content_location: String,

    /// The returned model.VolumeBackup instance.
    pub volume_backup: VolumeBackup,
}

impl CopyVolumeBackupResponse {
    /// Create a new CopyVolumeBackupResponse with required fields
    pub fn new(required: CopyVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            opc_work_request_id: required.opc_work_request_id,

            location: required.location,

            content_location: required.content_location,

            volume_backup: required.volume_backup,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: String) -> Self {
        self.opc_work_request_id = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: String) -> Self {
        self.location = value;
        self
    }

    /// Set content_location
    pub fn set_content_location(mut self, value: String) -> Self {
        self.content_location = value;
        self
    }

    /// Set volume_backup
    pub fn set_volume_backup(mut self, value: VolumeBackup) -> Self {
        self.volume_backup = value;
        self
    }
}
