use serde::{Deserialize, Serialize};

/// Instance configuration instance details.
/// This is a polymorphic type with instanceType as the discriminator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "instanceType")]
pub enum InstanceConfigurationInstanceDetails {
    /// Compute Instance Configuration instance details.
    #[serde(rename = "compute")]
    Compute(super::ComputeInstanceDetails),

    /// Multiple Compute Instance Configuration instance details.
    #[serde(rename = "instance_options")]
    InstanceOptions(super::ComputeInstanceOptions),
}

impl InstanceConfigurationInstanceDetails {
    /// Creates a new Compute variant
    pub fn compute(details: super::ComputeInstanceDetails) -> Self {
        Self::Compute(details)
    }

    /// Creates a new InstanceOptions variant
    pub fn instance_options(options: super::ComputeInstanceOptions) -> Self {
        Self::InstanceOptions(options)
    }
}
