use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComputeHostGroupResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Location of the resource.
    pub location: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// The returned model.ComputeHostGroup instance.
    pub compute_host_group: ComputeHostGroup,
}

/// Required fields for UpdateComputeHostGroupResponse
pub struct UpdateComputeHostGroupResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Location of the resource.
    pub location: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// The returned model.ComputeHostGroup instance.
    pub compute_host_group: ComputeHostGroup,
}

impl UpdateComputeHostGroupResponse {
    /// Create a new UpdateComputeHostGroupResponse with required fields
    pub fn new(required: UpdateComputeHostGroupResponseRequired) -> Self {
        Self {
            etag: required.etag,

            location: required.location,

            opc_request_id: required.opc_request_id,

            opc_work_request_id: required.opc_work_request_id,

            compute_host_group: required.compute_host_group,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: String) -> Self {
        self.location = value;
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

    /// Set compute_host_group
    pub fn set_compute_host_group(mut self, value: ComputeHostGroup) -> Self {
        self.compute_host_group = value;
        self
    }
}
