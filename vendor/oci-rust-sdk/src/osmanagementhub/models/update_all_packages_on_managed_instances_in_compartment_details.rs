use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update all packages of a specified type on managed instances within the specified compartment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAllPackagesOnManagedInstancesInCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// The types of updates to be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_types:
        Option<Vec<UpdateAllPackagesOnManagedInstancesInCompartmentDetailsUpdateTypes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for UpdateAllPackagesOnManagedInstancesInCompartmentDetails
pub struct UpdateAllPackagesOnManagedInstancesInCompartmentDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl UpdateAllPackagesOnManagedInstancesInCompartmentDetails {
    /// Create a new UpdateAllPackagesOnManagedInstancesInCompartmentDetails with required fields
    pub fn new(required: UpdateAllPackagesOnManagedInstancesInCompartmentDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            update_types: None,

            work_request_details: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set update_types
    pub fn set_update_types(
        mut self,
        value: Option<Vec<UpdateAllPackagesOnManagedInstancesInCompartmentDetailsUpdateTypes>>,
    ) -> Self {
        self.update_types = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set update_types (unwraps Option)
    pub fn with_update_types(
        mut self,
        value: Vec<UpdateAllPackagesOnManagedInstancesInCompartmentDetailsUpdateTypes>,
    ) -> Self {
        self.update_types = Some(value);
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
