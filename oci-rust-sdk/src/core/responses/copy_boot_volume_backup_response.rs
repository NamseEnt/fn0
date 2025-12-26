use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyBootVolumeBackupResponse {
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

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}


/// Required fields for CopyBootVolumeBackupResponse
pub struct CopyBootVolumeBackupResponseRequired {
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

    /// The returned model.BootVolumeBackup instance.
    pub boot_volume_backup: BootVolumeBackup,
}

impl CopyBootVolumeBackupResponse {
    /// Create a new CopyBootVolumeBackupResponse with required fields
    pub fn new(required: CopyBootVolumeBackupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            opc_work_request_id: required.opc_work_request_id,

            location: required.location,

            content_location: required.content_location,

            boot_volume_backup: required.boot_volume_backup,
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

    /// Set boot_volume_backup
    pub fn set_boot_volume_backup(mut self, value: BootVolumeBackup) -> Self {
        self.boot_volume_backup = value;
        self
    }
}


