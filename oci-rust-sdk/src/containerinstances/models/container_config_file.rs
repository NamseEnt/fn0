use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The file that is mounted on a container instance through a volume mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerConfigFile {
    /// The name of the file. The fileName should be unique across the volume.
    pub file_name: String,

    /// The base64 encoded contents of the file. The contents are decoded to plain text before mounted as a file to a container inside container instance.
    pub data: String,

    /// (Optional) Relative path for this file inside the volume mount directory. By default, the file is presented at the root of the volume mount path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Required fields for ContainerConfigFile
pub struct ContainerConfigFileRequired {
    /// The name of the file. The fileName should be unique across the volume.
    pub file_name: String,

    /// The base64 encoded contents of the file. The contents are decoded to plain text before mounted as a file to a container inside container instance.
    pub data: String,
}

impl ContainerConfigFile {
    /// Create a new ContainerConfigFile with required fields
    pub fn new(required: ContainerConfigFileRequired) -> Self {
        Self {
            file_name: required.file_name,

            data: required.data,

            path: None,
        }
    }

    /// Set file_name
    pub fn set_file_name(mut self, value: String) -> Self {
        self.file_name = value;
        self
    }

    /// Set data
    pub fn set_data(mut self, value: String) -> Self {
        self.data = value;
        self
    }

    /// Set path
    pub fn set_path(mut self, value: Option<String>) -> Self {
        self.path = value;
        self
    }

    /// Set path (unwraps Option)
    pub fn with_path(mut self, value: impl Into<String>) -> Self {
        self.path = Some(value.into());
        self
    }
}
