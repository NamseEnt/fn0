use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The list of supported platform configuration options for this shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapePlatformConfigOptions {
    /// The type of platform being configured.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ShapePlatformConfigOptionsType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure_boot_options: Option<ShapeSecureBootOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub measured_boot_options: Option<ShapeMeasuredBootOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_platform_module_options: Option<ShapeTrustedPlatformModuleOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket_platform_options: Option<ShapeNumaNodesPerSocketPlatformOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_encryption_options: Option<ShapeMemoryEncryptionOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub symmetric_multi_threading_options:
        Option<ShapeSymmetricMultiThreadingEnabledPlatformOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_control_service_options: Option<ShapeAccessControlServiceEnabledPlatformOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_instructions_options: Option<ShapeVirtualInstructionsEnabledPlatformOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_output_memory_management_unit_options:
        Option<ShapeInputOutputMemoryManagementUnitEnabledPlatformOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled_options: Option<PercentageOfCoresEnabledOptions>,
}

impl ShapePlatformConfigOptions {
    /// Create a new ShapePlatformConfigOptions
    pub fn new() -> Self {
        Self {
            r#type: None,

            secure_boot_options: None,

            measured_boot_options: None,

            trusted_platform_module_options: None,

            numa_nodes_per_socket_platform_options: None,

            memory_encryption_options: None,

            symmetric_multi_threading_options: None,

            access_control_service_options: None,

            virtual_instructions_options: None,

            input_output_memory_management_unit_options: None,

            percentage_of_cores_enabled_options: None,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<ShapePlatformConfigOptionsType>) -> Self {
        self.r#type = value;
        self
    }

    /// Set secure_boot_options
    pub fn set_secure_boot_options(mut self, value: Option<ShapeSecureBootOptions>) -> Self {
        self.secure_boot_options = value;
        self
    }

    /// Set measured_boot_options
    pub fn set_measured_boot_options(mut self, value: Option<ShapeMeasuredBootOptions>) -> Self {
        self.measured_boot_options = value;
        self
    }

    /// Set trusted_platform_module_options
    pub fn set_trusted_platform_module_options(
        mut self,
        value: Option<ShapeTrustedPlatformModuleOptions>,
    ) -> Self {
        self.trusted_platform_module_options = value;
        self
    }

    /// Set numa_nodes_per_socket_platform_options
    pub fn set_numa_nodes_per_socket_platform_options(
        mut self,
        value: Option<ShapeNumaNodesPerSocketPlatformOptions>,
    ) -> Self {
        self.numa_nodes_per_socket_platform_options = value;
        self
    }

    /// Set memory_encryption_options
    pub fn set_memory_encryption_options(
        mut self,
        value: Option<ShapeMemoryEncryptionOptions>,
    ) -> Self {
        self.memory_encryption_options = value;
        self
    }

    /// Set symmetric_multi_threading_options
    pub fn set_symmetric_multi_threading_options(
        mut self,
        value: Option<ShapeSymmetricMultiThreadingEnabledPlatformOptions>,
    ) -> Self {
        self.symmetric_multi_threading_options = value;
        self
    }

    /// Set access_control_service_options
    pub fn set_access_control_service_options(
        mut self,
        value: Option<ShapeAccessControlServiceEnabledPlatformOptions>,
    ) -> Self {
        self.access_control_service_options = value;
        self
    }

    /// Set virtual_instructions_options
    pub fn set_virtual_instructions_options(
        mut self,
        value: Option<ShapeVirtualInstructionsEnabledPlatformOptions>,
    ) -> Self {
        self.virtual_instructions_options = value;
        self
    }

    /// Set input_output_memory_management_unit_options
    pub fn set_input_output_memory_management_unit_options(
        mut self,
        value: Option<ShapeInputOutputMemoryManagementUnitEnabledPlatformOptions>,
    ) -> Self {
        self.input_output_memory_management_unit_options = value;
        self
    }

    /// Set percentage_of_cores_enabled_options
    pub fn set_percentage_of_cores_enabled_options(
        mut self,
        value: Option<PercentageOfCoresEnabledOptions>,
    ) -> Self {
        self.percentage_of_cores_enabled_options = value;
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: ShapePlatformConfigOptionsType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Set secure_boot_options (unwraps Option)
    pub fn with_secure_boot_options(mut self, value: ShapeSecureBootOptions) -> Self {
        self.secure_boot_options = Some(value);
        self
    }

    /// Set measured_boot_options (unwraps Option)
    pub fn with_measured_boot_options(mut self, value: ShapeMeasuredBootOptions) -> Self {
        self.measured_boot_options = Some(value);
        self
    }

    /// Set trusted_platform_module_options (unwraps Option)
    pub fn with_trusted_platform_module_options(
        mut self,
        value: ShapeTrustedPlatformModuleOptions,
    ) -> Self {
        self.trusted_platform_module_options = Some(value);
        self
    }

    /// Set numa_nodes_per_socket_platform_options (unwraps Option)
    pub fn with_numa_nodes_per_socket_platform_options(
        mut self,
        value: ShapeNumaNodesPerSocketPlatformOptions,
    ) -> Self {
        self.numa_nodes_per_socket_platform_options = Some(value);
        self
    }

    /// Set memory_encryption_options (unwraps Option)
    pub fn with_memory_encryption_options(mut self, value: ShapeMemoryEncryptionOptions) -> Self {
        self.memory_encryption_options = Some(value);
        self
    }

    /// Set symmetric_multi_threading_options (unwraps Option)
    pub fn with_symmetric_multi_threading_options(
        mut self,
        value: ShapeSymmetricMultiThreadingEnabledPlatformOptions,
    ) -> Self {
        self.symmetric_multi_threading_options = Some(value);
        self
    }

    /// Set access_control_service_options (unwraps Option)
    pub fn with_access_control_service_options(
        mut self,
        value: ShapeAccessControlServiceEnabledPlatformOptions,
    ) -> Self {
        self.access_control_service_options = Some(value);
        self
    }

    /// Set virtual_instructions_options (unwraps Option)
    pub fn with_virtual_instructions_options(
        mut self,
        value: ShapeVirtualInstructionsEnabledPlatformOptions,
    ) -> Self {
        self.virtual_instructions_options = Some(value);
        self
    }

    /// Set input_output_memory_management_unit_options (unwraps Option)
    pub fn with_input_output_memory_management_unit_options(
        mut self,
        value: ShapeInputOutputMemoryManagementUnitEnabledPlatformOptions,
    ) -> Self {
        self.input_output_memory_management_unit_options = Some(value);
        self
    }

    /// Set percentage_of_cores_enabled_options (unwraps Option)
    pub fn with_percentage_of_cores_enabled_options(
        mut self,
        value: PercentageOfCoresEnabledOptions,
    ) -> Self {
        self.percentage_of_cores_enabled_options = Some(value);
        self
    }
}

impl Default for ShapePlatformConfigOptions {
    fn default() -> Self {
        Self::new()
    }
}
