//! Registers projects as Signy tenants.
//!
//! Every project's telemetry is filed under a tenant named by its project id,
//! and Signy drops the data of a tenant nobody pushed a policy for. So a
//! project has to be registered before its first export arrives, and control
//! is the side that knows when projects come and go. Signy never calls out:
//! a failed push is control's to retry, which the cron reconcile does.

use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// The policy every project gets: 30 days for logs, traces and metrics alike,
/// and 512 MiB stored. Each signal is named rather than left to the default,
/// so a change to one of them is a change to this line and the reconcile below
/// notices a tenant that is still on an older policy.
pub const PROJECT_RETENTION: &str = "30d";
pub const PROJECT_LOG_RETENTION: &str = "30d";
pub const PROJECT_TRACE_RETENTION: &str = "30d";
pub const PROJECT_METRIC_RETENTION: &str = "30d";
pub const PROJECT_MAX_STORED_BYTES: &str = "512MiB";

/// Pushed when a project is deleted. Retention `0` is how Signy deletes a
/// tenant's data; the zero storage limit refuses anything still in flight.
const DELETED_RETENTION: &str = "0";
const DELETED_MAX_STORED_BYTES: &str = "0";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TenantPolicy {
    pub retention: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_stored_bytes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_retention: Option<String>,
}

#[derive(Deserialize)]
struct TenantListResponse {
    tenants: Vec<TenantListEntry>,
}

#[derive(Deserialize)]
struct TenantListEntry {
    tenant: String,
    retention: String,
    #[serde(default)]
    max_stored_bytes: Option<String>,
    #[serde(default)]
    log_retention: Option<String>,
    #[serde(default)]
    trace_retention: Option<String>,
    #[serde(default)]
    metric_retention: Option<String>,
}

pub fn project_policy() -> TenantPolicy {
    TenantPolicy {
        retention: PROJECT_RETENTION.to_string(),
        max_stored_bytes: Some(PROJECT_MAX_STORED_BYTES.to_string()),
        log_retention: Some(PROJECT_LOG_RETENTION.to_string()),
        trace_retention: Some(PROJECT_TRACE_RETENTION.to_string()),
        metric_retention: Some(PROJECT_METRIC_RETENTION.to_string()),
    }
}

fn deleted_project_policy() -> TenantPolicy {
    TenantPolicy {
        retention: DELETED_RETENTION.to_string(),
        max_stored_bytes: Some(DELETED_MAX_STORED_BYTES.to_string()),
        log_retention: None,
        trace_retention: None,
        metric_retention: None,
    }
}

struct SignyAdmin {
    url: String,
    access_client_id: String,
    access_client_secret: String,
}

impl SignyAdmin {
    fn from_env() -> anyhow::Result<Self> {
        let read = |name: &str| std::env::var(name).map_err(|_| anyhow::anyhow!("{name} not set"));
        Ok(Self {
            url: read("FN0_SIGNY_URL")?.trim_end_matches('/').to_string(),
            access_client_id: read("FN0_SIGNY_ACCESS_CLIENT_ID")?,
            access_client_secret: read("FN0_SIGNY_ACCESS_CLIENT_SECRET")?,
        })
    }

    async fn send(&self, method: &str, path: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let request = http::Request::builder()
            .uri(format!("{}/signy/api/v1/admin/{path}", self.url))
            .method(method)
            .header("CF-Access-Client-Id", &self.access_client_id)
            .header("CF-Access-Client-Secret", &self.access_client_secret)
            .header("Content-Type", "application/json")
            .body(body)?;
        let response = http::Client::new().send(request).await?;
        let status = response.status();
        let body = response.into_body().bytes().await?.to_vec();
        if !status.is_success() {
            anyhow::bail!(
                "signy admin {method} {path} answered {status}: {}",
                String::from_utf8_lossy(&body)
            );
        }
        Ok(body)
    }

    async fn put_policy(&self, tenant: &str, policy: &TenantPolicy) -> anyhow::Result<()> {
        self.send(
            "PUT",
            &format!("tenants/{tenant}/retention"),
            serde_json::to_vec(policy)?,
        )
        .await?;
        Ok(())
    }
}

pub async fn register_project(project_id: &str) -> anyhow::Result<()> {
    SignyAdmin::from_env()?
        .put_policy(project_id, &project_policy())
        .await
}

pub async fn delete_project(project_id: &str) -> anyhow::Result<()> {
    SignyAdmin::from_env()?
        .put_policy(project_id, &deleted_project_policy())
        .await
}

pub struct ReconcileStats {
    pub registered_projects: u64,
}

/// Pushes the project policy for every project that lacks it or holds an
/// outdated one. A project already at retention `0` is being deleted and is
/// never pushed back: undoing a deletion is not a reconcile's decision.
pub async fn reconcile_projects(project_ids: &[String]) -> anyhow::Result<ReconcileStats> {
    let admin = SignyAdmin::from_env()?;
    let listing: TenantListResponse =
        serde_json::from_slice(&admin.send("GET", "tenants", Vec::new()).await?)?;
    let existing: std::collections::HashMap<String, TenantPolicy> = listing
        .tenants
        .into_iter()
        .map(|entry| {
            (
                entry.tenant,
                TenantPolicy {
                    retention: entry.retention,
                    max_stored_bytes: entry.max_stored_bytes,
                    log_retention: entry.log_retention,
                    trace_retention: entry.trace_retention,
                    metric_retention: entry.metric_retention,
                },
            )
        })
        .collect();
    let mut registered_projects = 0;
    for project_id in projects_needing_policy(project_ids, &existing) {
        admin.put_policy(project_id, &project_policy()).await?;
        registered_projects += 1;
    }
    Ok(ReconcileStats {
        registered_projects,
    })
}

fn projects_needing_policy<'a>(
    project_ids: &'a [String],
    existing: &std::collections::HashMap<String, TenantPolicy>,
) -> Vec<&'a str> {
    let wanted = project_policy();
    project_ids
        .iter()
        .filter(|project_id| match existing.get(project_id.as_str()) {
            None => true,
            Some(policy) if policy.retention == DELETED_RETENTION => false,
            Some(policy) => *policy != wanted,
        })
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        PROJECT_MAX_STORED_BYTES, PROJECT_RETENTION, TenantPolicy, deleted_project_policy,
        project_policy, projects_needing_policy,
    };

    #[test]
    fn registers_missing_and_outdated_projects_but_never_revives_a_deleted_one() {
        let project_ids = vec![
            "missing".to_string(),
            "current".to_string(),
            "outdated".to_string(),
            "deleted".to_string(),
        ];
        let existing = std::collections::HashMap::from([
            ("current".to_string(), project_policy()),
            (
                "outdated".to_string(),
                TenantPolicy {
                    retention: "7d".to_string(),
                    max_stored_bytes: None,
                    log_retention: None,
                    trace_retention: None,
                    metric_retention: None,
                },
            ),
            ("deleted".to_string(), deleted_project_policy()),
            ("fn0".to_string(), project_policy()),
        ]);

        assert_eq!(
            projects_needing_policy(&project_ids, &existing),
            vec!["missing", "outdated"]
        );
    }

    /// A tenant already registered under the policy that named no signal is
    /// reconciled onto this one, which is what carries the signal periods to
    /// the projects registered before they existed.
    #[test]
    fn a_policy_without_signal_periods_is_outdated() {
        let existing = std::collections::HashMap::from([(
            "pre-signal".to_string(),
            TenantPolicy {
                retention: PROJECT_RETENTION.to_string(),
                max_stored_bytes: Some(PROJECT_MAX_STORED_BYTES.to_string()),
                log_retention: None,
                trace_retention: None,
                metric_retention: None,
            },
        )]);
        assert_eq!(
            projects_needing_policy(&["pre-signal".to_string()], &existing),
            vec!["pre-signal"]
        );
    }

    #[test]
    fn the_pushed_body_is_the_whole_policy() {
        assert_eq!(
            serde_json::to_value(project_policy()).unwrap(),
            serde_json::json!({
                "retention": "30d",
                "max_stored_bytes": "512MiB",
                "log_retention": "30d",
                "trace_retention": "30d",
                "metric_retention": "30d"
            })
        );
    }
}
