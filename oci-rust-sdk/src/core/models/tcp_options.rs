use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional and valid only for TCP. Use to specify particular destination ports for TCP rules. If you specify TCP as the protocol but omit this object, then all destination ports are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port_range: Option<PortRange>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port_range: Option<PortRange>,
}

impl TcpOptions {
    /// Create a new TcpOptions
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

impl Default for TcpOptions {
    fn default() -> Self {
        Self::new()
    }
}
