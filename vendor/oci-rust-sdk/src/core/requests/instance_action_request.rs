use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceActionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,

    /// The action to perform on the instance.
    pub action: String,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Instance Power Action details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_power_action_details: Option<ResetActionDetails>,
}

/// Required fields for InstanceActionRequest
pub struct InstanceActionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,

    /// The action to perform on the instance.
    pub action: String,
}

impl InstanceActionRequest {
    /// Create a new InstanceActionRequest with required fields
    pub fn new(required: InstanceActionRequestRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            action: required.action,

            opc_retry_token: None,

            if_match: None,

            instance_power_action_details: None,
        }
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set action
    pub fn set_action(mut self, value: String) -> Self {
        self.action = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set instance_power_action_details
    pub fn set_instance_power_action_details(mut self, value: Option<ResetActionDetails>) -> Self {
        self.instance_power_action_details = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set instance_power_action_details (unwraps Option)
    pub fn with_instance_power_action_details(mut self, value: ResetActionDetails) -> Self {
        self.instance_power_action_details = Some(value);
        self
    }
}
