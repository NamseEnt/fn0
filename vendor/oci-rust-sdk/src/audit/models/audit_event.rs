use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// All the attributes of an audit event. For more information, see [Viewing Audit Log Events](https://docs.oracle.com/iaas/Content/Audit/Tasks/viewinglogevents.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    /// The type of event that happened. <p> The service that produces the event can also add, remove, or change the meaning of a field. A service implementing these type changes would publish a new version of an {@code eventType} and revise the {@code eventTypeVersion} field. <p> Example: {@code com.oraclecloud.ComputeApi.GetInstance}
    pub event_type: String,

    /// The version of the CloudEvents specification. The structure of the envelope follows the [CloudEvents](https://github.com/cloudevents/spec) industry standard format hosted by the [Cloud Native Computing Foundation ( CNCF)](https://www.cncf.io/). <p> Audit uses version 0.1 specification of the CloudEvents event envelope. <p> Example: {@code 0.1}
    pub cloud_events_version: String,

    /// The version of the event type. This version applies to the payload of the event, not the envelope. Use {@code cloudEventsVersion} to determine the version of the envelope. <p> Example: {@code 2.0}
    pub event_type_version: String,

    /// The source of the event. <p> Example: {@code ComputeApi}
    pub source: String,

    /// The GUID of the event.
    pub event_id: String,

    /// The time the event occurred, expressed in [RFC 3339](https://tools.ietf.org/html/rfc3339) timestamp format. <p> Example: {@code 2019-09-18T00:10:59.252Z}
    pub event_time: DateTime<Utc>,

    /// The content type of the data contained in {@code data}. <p> Example: {@code application/json}
    pub content_type: String,

    pub data: Data,
}

/// Required fields for AuditEvent
pub struct AuditEventRequired {
    /// The type of event that happened. <p> The service that produces the event can also add, remove, or change the meaning of a field. A service implementing these type changes would publish a new version of an {@code eventType} and revise the {@code eventTypeVersion} field. <p> Example: {@code com.oraclecloud.ComputeApi.GetInstance}
    pub event_type: String,

    /// The version of the CloudEvents specification. The structure of the envelope follows the [CloudEvents](https://github.com/cloudevents/spec) industry standard format hosted by the [Cloud Native Computing Foundation ( CNCF)](https://www.cncf.io/). <p> Audit uses version 0.1 specification of the CloudEvents event envelope. <p> Example: {@code 0.1}
    pub cloud_events_version: String,

    /// The version of the event type. This version applies to the payload of the event, not the envelope. Use {@code cloudEventsVersion} to determine the version of the envelope. <p> Example: {@code 2.0}
    pub event_type_version: String,

    /// The source of the event. <p> Example: {@code ComputeApi}
    pub source: String,

    /// The GUID of the event.
    pub event_id: String,

    /// The time the event occurred, expressed in [RFC 3339](https://tools.ietf.org/html/rfc3339) timestamp format. <p> Example: {@code 2019-09-18T00:10:59.252Z}
    pub event_time: DateTime<Utc>,

    /// The content type of the data contained in {@code data}. <p> Example: {@code application/json}
    pub content_type: String,

    pub data: Data,
}

impl AuditEvent {
    /// Create a new AuditEvent with required fields
    pub fn new(required: AuditEventRequired) -> Self {
        Self {
            event_type: required.event_type,

            cloud_events_version: required.cloud_events_version,

            event_type_version: required.event_type_version,

            source: required.source,

            event_id: required.event_id,

            event_time: required.event_time,

            content_type: required.content_type,

            data: required.data,
        }
    }

    /// Set event_type
    pub fn set_event_type(mut self, value: String) -> Self {
        self.event_type = value;
        self
    }

    /// Set cloud_events_version
    pub fn set_cloud_events_version(mut self, value: String) -> Self {
        self.cloud_events_version = value;
        self
    }

    /// Set event_type_version
    pub fn set_event_type_version(mut self, value: String) -> Self {
        self.event_type_version = value;
        self
    }

    /// Set source
    pub fn set_source(mut self, value: String) -> Self {
        self.source = value;
        self
    }

    /// Set event_id
    pub fn set_event_id(mut self, value: String) -> Self {
        self.event_id = value;
        self
    }

    /// Set event_time
    pub fn set_event_time(mut self, value: DateTime<Utc>) -> Self {
        self.event_time = value;
        self
    }

    /// Set content_type
    pub fn set_content_type(mut self, value: String) -> Self {
        self.content_type = value;
        self
    }

    /// Set data
    pub fn set_data(mut self, value: Data) -> Self {
        self.data = value;
        self
    }
}
