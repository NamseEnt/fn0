use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A resource created or operated on by a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestResource {
    /// The resource type that the work request affects.
    pub entity_type: TargetResourceEntityType,

    /// The way in which this resource is affected by the work tracked in the work request. A resource being created, updated, or deleted will remain in the IN_PROGRESS state until work is complete for that resource at which point it will transition to CREATED, UPDATED, or DELETED, respectively.
    pub action_type: ActionType,

    /// The identifier of the resource the work request affects.
    pub identifier: String,

    /// The URI path that the user can do a GET on to access the resource metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_uri: Option<String>,

    /// The name of the resource. Not all resources will have a name specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Additional information that helps to explain the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Required fields for WorkRequestResource
pub struct WorkRequestResourceRequired {
    /// The resource type that the work request affects.
    pub entity_type: TargetResourceEntityType,

    /// The way in which this resource is affected by the work tracked in the work request. A resource being created, updated, or deleted will remain in the IN_PROGRESS state until work is complete for that resource at which point it will transition to CREATED, UPDATED, or DELETED, respectively.
    pub action_type: ActionType,

    /// The identifier of the resource the work request affects.
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

            name: None,

            metadata: None,
        }
    }

    /// Set entity_type
    pub fn set_entity_type(mut self, value: TargetResourceEntityType) -> Self {
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

    /// Set name
    pub fn set_name(mut self, value: Option<String>) -> Self {
        self.name = value;
        self
    }

    /// Set metadata
    pub fn set_metadata(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.metadata = value;
        self
    }

    /// Set entity_uri (unwraps Option)
    pub fn with_entity_uri(mut self, value: impl Into<String>) -> Self {
        self.entity_uri = Some(value.into());
        self
    }

    /// Set name (unwraps Option)
    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Set metadata (unwraps Option)
    pub fn with_metadata(mut self, value: HashMap<String, String>) -> Self {
        self.metadata = Some(value);
        self
    }
}
