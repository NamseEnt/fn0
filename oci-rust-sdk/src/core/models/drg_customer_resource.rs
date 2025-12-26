use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The IPSEC / FC / RPC info returned in DrgCustomerResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgCustomerResource {
    /// OCID of the IPSEC / FC / RPC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The friendly name of the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// the compartment id of the DRG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// the lifeCycleState of the IPSEC / FC / RPC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl DrgCustomerResource {
    /// Create a new DrgCustomerResource
    pub fn new() -> Self {
        Self {
            id: None,

            display_name: None,

            compartment_id: None,

            state: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set state
    pub fn set_state(mut self, value: Option<String>) -> Self {
        self.state = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set state (unwraps Option)
    pub fn with_state(mut self, value: impl Into<String>) -> Self {
        self.state = Some(value.into());
        self
    }
}

impl Default for DrgCustomerResource {
    fn default() -> Self {
        Self::new()
    }
}
