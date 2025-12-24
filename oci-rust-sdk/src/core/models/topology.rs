use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the representation of a virtual network topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topology {
    /// Lists entities comprising the virtual network topology.
    pub entities: Vec<serde_json::Value>,

    /// Lists relationships between entities in the virtual network topology.
    pub relationships: Vec<TopologyEntityRelationship>,

    /// Lists entities that are limited during ingestion. The values for the items in the list are the entity type names of the limitedEntities. Example: {@code vcn}
    pub limited_entities: Vec<String>,

    /// Records when the virtual network topology was created, in [RFC3339](https://tools.ietf.org/html/rfc3339) format for date and time.
    pub time_created: DateTime<Utc>,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for Topology
pub struct TopologyRequired {
    /// Lists entities comprising the virtual network topology.
    pub entities: Vec<serde_json::Value>,

    /// Lists relationships between entities in the virtual network topology.
    pub relationships: Vec<TopologyEntityRelationship>,

    /// Lists entities that are limited during ingestion. The values for the items in the list are the entity type names of the limitedEntities. Example: {@code vcn}
    pub limited_entities: Vec<String>,

    /// Records when the virtual network topology was created, in [RFC3339](https://tools.ietf.org/html/rfc3339) format for date and time.
    pub time_created: DateTime<Utc>,

    pub r#type: String,
}

impl Topology {
    /// Create a new Topology with required fields
    pub fn new(required: TopologyRequired) -> Self {
        Self {
            entities: required.entities,

            relationships: required.relationships,

            limited_entities: required.limited_entities,

            time_created: required.time_created,

            r#type: required.r#type,
        }
    }

    /// Set entities
    pub fn set_entities(mut self, value: Vec<serde_json::Value>) -> Self {
        self.entities = value;
        self
    }

    /// Set relationships
    pub fn set_relationships(mut self, value: Vec<TopologyEntityRelationship>) -> Self {
        self.relationships = value;
        self
    }

    /// Set limited_entities
    pub fn set_limited_entities(mut self, value: Vec<String>) -> Self {
        self.limited_entities = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
