use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeCapacityReportResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityReport instance.
    pub compute_capacity_report: ComputeCapacityReport,
}


/// Required fields for CreateComputeCapacityReportResponse
pub struct CreateComputeCapacityReportResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeCapacityReport instance.
    pub compute_capacity_report: ComputeCapacityReport,
}

impl CreateComputeCapacityReportResponse {
    /// Create a new CreateComputeCapacityReportResponse with required fields
    pub fn new(required: CreateComputeCapacityReportResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            compute_capacity_report: required.compute_capacity_report,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set compute_capacity_report
    pub fn set_compute_capacity_report(mut self, value: ComputeCapacityReport) -> Self {
        self.compute_capacity_report = value;
        self
    }
}


