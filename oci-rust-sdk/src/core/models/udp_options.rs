use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional and valid only for UDP. Use to specify particular destination ports for UDP rules. If you specify UDP as the protocol but omit this object, then all destination ports are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port_range: Option<PortRange>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port_range: Option<PortRange>,
}

impl UdpOptions {
    /// Create a new UdpOptions
    pub fn new() -> Self {
        Self {
            destination_port_range: None,

            source_port_range: None,
        }
    }

    /// Set destination_port_range
    pub fn set_destination_port_range(mut self, value: Option<PortRange>) -> Self {
        self.destination_port_range = value;
        self
    }

    /// Set source_port_range
    pub fn set_source_port_range(mut self, value: Option<PortRange>) -> Self {
        self.source_port_range = value;
        self
    }

    /// Set destination_port_range (unwraps Option)
    pub fn with_destination_port_range(mut self, value: PortRange) -> Self {
        self.destination_port_range = Some(value);
        self
    }

    /// Set source_port_range (unwraps Option)
    pub fn with_source_port_range(mut self, value: PortRange) -> Self {
        self.source_port_range = Some(value);
        self
    }
}

impl Default for UdpOptions {
    fn default() -> Self {
        Self::new()
    }
}
