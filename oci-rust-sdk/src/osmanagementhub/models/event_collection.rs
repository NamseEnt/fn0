use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of events returned for the {@link #listEvents(ListEventsRequest) listEvents} operation. The list contains a summary of each event and other information, such as metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventCollection {
    /// List of events.
    pub items: Vec<EventSummary>,
}

/// Required fields for EventCollection
pub struct EventCollectionRequired {
    /// List of events.
    pub items: Vec<EventSummary>,
}

impl EventCollection {
    /// Create a new EventCollection with required fields
    pub fn new(required: EventCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<EventSummary>) -> Self {
        self.items = value;
        self
    }
}
