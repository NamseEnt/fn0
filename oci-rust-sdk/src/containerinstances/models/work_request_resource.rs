use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A resource created or operated on by a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestResource {
    /// The resource type the work request affects.
    pub entity_type: String,

    /// The way in which this resource is affected by the work tracked in the work request. A resource being created, updated, or deleted remains in the IN_PROGRESS state until work is complete for that resource, at which point it updates to CREATED, UPDATED, or DELETED, respectively.
    pub action_type: ActionType,

    /// The ID of the resource the work request affects.
    pub identifier: String,

    /// The URI path that the user can do a GET on to access the resource metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_uri: Option<String>,
}

/// Required fields for WorkRequestResource
pub struct WorkRequestResourceRequired {
    /// The resource type the work request affects.
    pub entity_type: String,

    /// The way in which this resource is affected by the work tracked in the work request. A resource being created, updated, or deleted remains in the IN_PROGRESS state until work is complete for that resource, at which point it updates to CREATED, UPDATED, or DELETED, respectively.
    pub action_type: ActionType,

    /// The ID of the resource the work request affects.
    pub identifier: String,
}

impl WorkRequestResource {
    /// Create a new WorkRequestResource with required fields
    pub fn new(required: WorkRequestResourceRequired) -> Self {
        Self {
            entity_type: required.entity_type,

            action_type: required.action_type,

            identifier: required.identifier,

            entity_uri: None,
        }
    }

    /// Set entity_type
    pub fn set_entity_type(mut self, value: String) -> Self {
        self.entity_type = value;
        self
    }

    /// Set action_type
    pub fn set_action_type(mut self, value: ActionType) -> Self {
        self.action_type = value;
        self
    }

    /// Set identifier
    pub fn set_identifier(mut self, value: String) -> Self {
        self.identifier = value;
        self
    }

    /// Set entity_uri
    pub fn set_entity_uri(mut self, value: Option<String>) -> Self {
        self.entity_uri = value;
        self
    }

    /// Set entity_uri (unwraps Option)
    pub fn with_entity_uri(mut self, value: impl Into<String>) -> Self {
        self.entity_uri = Some(value.into());
        self
    }
}
