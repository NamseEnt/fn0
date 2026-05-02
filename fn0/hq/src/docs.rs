use forte_macros::forte_doc;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

pub use doc_db::DbRequest;

// === Host Status ===

#[forte_doc]
pub struct HostStatusDoc {
    #[pk]
    pub site_name: String,
    #[sk]
    pub host_id: String,
    pub addr: String,
    pub dns_addr: Option<String>,
    pub generation: Option<u64>,
    pub instances: Option<u64>,
    pub healthy: bool,
    pub consecutive_failures: u32,
    pub last_verified_at: Option<String>,
    pub graceful: bool,
    pub graceful_purpose: Option<GracefulPurpose>,
    pub graceful_since_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GracefulPurpose {
    Terminate,
    ImageSwap,
}

// === Scale Config ===

#[forte_doc]
pub struct ScaleConfigDoc {
    #[pk]
    pub site_name: String,
    pub instances_per_gb: NonZeroUsize,
    pub instances_per_core: NonZeroUsize,
    pub scale_out_threshold_percent: NonZeroUsize,
    pub scale_in_threshold_percent: NonZeroUsize,
    pub scale_out_cooldown_secs: NonZeroUsize,
    pub scale_in_threshold_ticks: NonZeroUsize,
    pub scale_in_cooldown_secs: NonZeroUsize,
    pub max_hosts: NonZeroUsize,
    pub min_hosts: NonZeroUsize,
}

// === Worker Target / Cwasm Ready / Last Stable ===

#[forte_doc]
pub struct WorkerTargetDoc {
    #[pk]
    pub site_name: String,
    pub image_registry: String,
    pub image_repository: String,
    pub image_tag: String,
    pub fn0_wasmtime_version: String,
}

#[forte_doc]
pub struct CwasmReadyDoc {
    #[pk]
    pub site_name: String,
    pub fn0_wasmtime_version: String,
}

#[forte_doc]
pub struct WorkerLastStableDoc {
    #[pk]
    pub site_name: String,
    pub image_registry: String,
    pub image_repository: String,
    pub image_tag: String,
    pub fn0_wasmtime_version: String,
}

// === Deploy Job ===

#[forte_doc]
pub struct DeployJobDoc {
    #[sk]
    pub job_id: String,
    pub subdomain: String,
    pub code_id: u64,
    pub build_id: Option<String>,
    pub env_ciphertext: Option<String>,
    pub phase: DeployJobPhase,
    pub code_version: Option<u64>,
    pub old_build_ids: Option<Vec<String>>,
    pub generation: Option<u64>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeployJobPhase {
    Queued,
    EnvUploaded,
    CwasmCompiled,
    DbEnsured,
    Versioned,
    Deployed,
    BuildRegistered,
    R2GcEnqueued,
    Pushed,
    Done,
    Failed,
}

impl DeployJobDoc {
    pub fn is_terminal(&self) -> bool {
        matches!(self.phase, DeployJobPhase::Done | DeployJobPhase::Failed)
    }
}

// === Rename Job ===

#[forte_doc]
pub struct RenameJobDoc {
    #[sk]
    pub job_id: String,
    pub github_username: String,
    pub old_project_name: String,
    pub new_project_name: String,
    pub old_subdomain: String,
    pub new_subdomain: String,
    pub code_id: u64,
    pub code_version: Option<u64>,
    pub build_id: Option<String>,
    pub env_ciphertext: Option<String>,
    pub old_build_ids: Option<Vec<String>>,
    pub generation: Option<u64>,
    pub phase: RenameJobPhase,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenameJobPhase {
    Queued,
    OldWritesBlocked,
    NewDbSeeded,
    NewEnvUploaded,
    NewCwasmCompiled,
    NewVersioned,
    NewDeploymentInserted,
    NewBuildRegistered,
    NewDeploymentPushed,
    NewDeploymentVerified,
    ProjectRenamed,
    OldUndeployed,
    OldEnvDeleted,
    OldDbDestroyed,
    Done,
    Failed,
}

impl RenameJobDoc {
    pub fn is_terminal(&self) -> bool {
        matches!(self.phase, RenameJobPhase::Done | RenameJobPhase::Failed)
    }
}

// === Project / Code Version / Next Code ID ===

#[forte_doc]
pub struct ProjectDoc {
    #[pk]
    pub github_username: String,
    #[pk]
    pub project_name: String,
    pub code_id: u64,
}

#[forte_doc]
pub struct CodeVersionDoc {
    #[pk]
    pub code_id: u64,
    pub count: u64,
}

#[forte_doc]
pub struct NextCodeIdDoc {
    pub count: u64,
}

// === Forte Build Registry ===

#[forte_doc]
pub struct ForteBuildDoc {
    #[pk]
    pub subdomain: String,
    pub build_id: String,
}

// === Deployment ===

#[forte_doc]
pub struct DeploymentDoc {
    #[sk]
    pub seq: u64,
    pub subdomain: String,
    pub kind: DeploymentKind,
    pub code_id: Option<u64>,
    pub code_version: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentKind {
    Deploy,
    Undeploy,
}

#[forte_doc]
pub struct NextDeploymentSeqDoc {
    pub count: u64,
}

// === Custom Domain ===

#[forte_doc]
pub struct CustomDomainBySubdomainDoc {
    #[sk]
    pub subdomain: String,
    pub domain: String,
}

#[forte_doc]
pub struct CustomDomainByDomainDoc {
    #[sk]
    pub domain: String,
    pub subdomain: String,
    pub cloudflare_hostname_id: String,
}

#[forte_doc]
pub struct CustomDomainAddJobDoc {
    #[sk]
    pub job_id: String,
    pub github_username: String,
    pub project_name: String,
    pub subdomain: String,
    pub domain: String,
    pub cloudflare_hostname_id: Option<String>,
    pub phase: CustomDomainAddPhase,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomDomainAddPhase {
    Queued,
    CloudflareHostnameCreated,
    DocPersisted,
    Pushed,
    Verified,
    Done,
    Failed,
}

impl CustomDomainAddJobDoc {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.phase,
            CustomDomainAddPhase::Done | CustomDomainAddPhase::Failed
        )
    }
}

#[forte_doc]
pub struct CustomDomainRemoveJobDoc {
    #[sk]
    pub job_id: String,
    pub github_username: String,
    pub project_name: String,
    pub subdomain: String,
    pub domain: String,
    pub cloudflare_hostname_id: Option<String>,
    pub phase: CustomDomainRemovePhase,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomDomainRemovePhase {
    Queued,
    DocDeleted,
    Pushed,
    CloudflareHostnameDeleted,
    Done,
    Failed,
}

impl CustomDomainRemoveJobDoc {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.phase,
            CustomDomainRemovePhase::Done | CustomDomainRemovePhase::Failed
        )
    }
}

// === Hq Jobs (queue) ===

#[forte_doc]
pub struct HqJobPendingDoc {
    #[sk]
    pub created_at: String,
    #[sk]
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[forte_doc]
pub struct HqJobProcessingDoc {
    #[sk]
    pub updated_at: String,
    #[sk]
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub retry_count: u32,
    pub max_retries: u32,
    pub original_created_at: String,
}

#[forte_doc]
pub struct HqJobDeadDoc {
    #[sk]
    pub died_at: String,
    #[sk]
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub error_message: String,
    pub retry_count: u32,
    pub original_created_at: String,
}
