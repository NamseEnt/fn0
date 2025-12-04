use serde::{Deserialize, Serialize};

/// Volume attachment details for instance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InstanceConfigurationAttachVolumeDetails {
    Iscsi(super::InstanceConfigurationIscsiAttachVolumeDetails),
    Paravirtualized(super::InstanceConfigurationParavirtualizedAttachVolumeDetails),
}

impl InstanceConfigurationAttachVolumeDetails {
    pub fn iscsi(details: super::InstanceConfigurationIscsiAttachVolumeDetails) -> Self {
        Self::Iscsi(details)
    }

    pub fn paravirtualized(
        details: super::InstanceConfigurationParavirtualizedAttachVolumeDetails,
    ) -> Self {
        Self::Paravirtualized(details)
    }
}
