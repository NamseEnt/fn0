use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The Letter of Authority for the cross-connect. You must submit this letter when requesting cabling for the cross-connect at the FastConnect location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LetterOfAuthority {
    /// The name of the entity authorized by this Letter of Authority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_entity_name: Option<String>,

    /// The type of cross-connect fiber, termination, and optical specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_type: Option<LetterOfAuthorityCircuitType>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_id: Option<String>,

    /// The address of the FastConnect location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility_location: Option<String>,

    /// The meet-me room port for this cross-connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_name: Option<String>,

    /// The date and time when the Letter of Authority expires, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_expires: Option<DateTime<Utc>>,

    /// The date and time the Letter of Authority was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_issued: Option<DateTime<Utc>>,
}

impl LetterOfAuthority {
    /// Create a new LetterOfAuthority
    pub fn new() -> Self {
        Self {
            authorized_entity_name: None,

            circuit_type: None,

            cross_connect_id: None,

            facility_location: None,

            port_name: None,

            time_expires: None,

            time_issued: None,
        }
    }

    /// Set authorized_entity_name
    pub fn set_authorized_entity_name(mut self, value: Option<String>) -> Self {
        self.authorized_entity_name = value;
        self
    }

    /// Set circuit_type
    pub fn set_circuit_type(mut self, value: Option<LetterOfAuthorityCircuitType>) -> Self {
        self.circuit_type = value;
        self
    }

    /// Set cross_connect_id
    pub fn set_cross_connect_id(mut self, value: Option<String>) -> Self {
        self.cross_connect_id = value;
        self
    }

    /// Set facility_location
    pub fn set_facility_location(mut self, value: Option<String>) -> Self {
        self.facility_location = value;
        self
    }

    /// Set port_name
    pub fn set_port_name(mut self, value: Option<String>) -> Self {
        self.port_name = value;
        self
    }

    /// Set time_expires
    pub fn set_time_expires(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_expires = value;
        self
    }

    /// Set time_issued
    pub fn set_time_issued(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_issued = value;
        self
    }

    /// Set authorized_entity_name (unwraps Option)
    pub fn with_authorized_entity_name(mut self, value: impl Into<String>) -> Self {
        self.authorized_entity_name = Some(value.into());
        self
    }

    /// Set circuit_type (unwraps Option)
    pub fn with_circuit_type(mut self, value: LetterOfAuthorityCircuitType) -> Self {
        self.circuit_type = Some(value);
        self
    }

    /// Set cross_connect_id (unwraps Option)
    pub fn with_cross_connect_id(mut self, value: impl Into<String>) -> Self {
        self.cross_connect_id = Some(value.into());
        self
    }

    /// Set facility_location (unwraps Option)
    pub fn with_facility_location(mut self, value: impl Into<String>) -> Self {
        self.facility_location = Some(value.into());
        self
    }

    /// Set port_name (unwraps Option)
    pub fn with_port_name(mut self, value: impl Into<String>) -> Self {
        self.port_name = Some(value.into());
        self
    }

    /// Set time_expires (unwraps Option)
    pub fn with_time_expires(mut self, value: DateTime<Utc>) -> Self {
        self.time_expires = Some(value);
        self
    }

    /// Set time_issued (unwraps Option)
    pub fn with_time_issued(mut self, value: DateTime<Utc>) -> Self {
        self.time_issued = Some(value);
        self
    }
}

impl Default for LetterOfAuthority {
    fn default() -> Self {
        Self::new()
    }
}
