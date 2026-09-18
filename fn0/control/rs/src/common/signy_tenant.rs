//! Control-owned registration of project tenants in Signy.
//!
//! ProjectDoc is the source of truth. This module only transports the latest
//! document value to Signy through the durable queue; it never invents a
//! policy, reconciles from a platform default, or changes retention on delete.

use crate::docs::TelemetryPolicy;
use forte_sdk::*;
use serde::Deserialize;

pub struct ReconcileStats {
    pub requeued_records: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicySyncResult {
    Applied,
    Stale,
}

#[derive(Deserialize)]
struct TenantPolicyResponse {
    #[serde(default)]
    revision: u64,
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
}

pub async fn register_project(
    project_id: &str,
    policy: &TelemetryPolicy,
) -> anyhow::Result<PolicySyncResult> {
    let admin = SignyAdmin::from_env()?;
    let requested_policy = serde_json::json!({
        "revision": policy.revision,
        "retention": policy.base_retention,
        "log_retention": policy.log_retention_override,
        "trace_retention": policy.trace_retention_override,
        "metric_retention": policy.metric_retention_override,
        "max_stored_bytes": policy.max_stored_bytes,
    });
    let response: TenantPolicyResponse = serde_json::from_slice(
        &admin
            .send(
                "PUT",
                &format!("project-tenants/{project_id}/retention"),
                serde_json::to_vec(&requested_policy)?,
            )
            .await?,
    )?;
    if response.revision < policy.revision {
        anyhow::bail!(
            "Signy returned revision {} older than ProjectDoc revision {}",
            response.revision,
            policy.revision
        );
    }
    if response.revision > policy.revision {
        return Ok(PolicySyncResult::Stale);
    }
    let response_policy = serde_json::json!({
        "revision": response.revision,
        "retention": response.retention,
        "log_retention": response.log_retention,
        "trace_retention": response.trace_retention,
        "metric_retention": response.metric_retention,
        "max_stored_bytes": response.max_stored_bytes,
    });
    if response_policy != requested_policy {
        anyhow::bail!("Signy returned a policy different from ProjectDoc");
    }
    Ok(PolicySyncResult::Applied)
}

/// Reads the policy already registered in Signy for migration. Missing
/// policies are returned as `None`; callers record that as an operator
/// exception rather than inventing a value.
pub async fn read_policy(project_id: &str) -> anyhow::Result<Option<TelemetryPolicy>> {
    let admin = SignyAdmin::from_env()?;
    let request = http::Request::builder()
        .uri(format!(
            "{}/signy/api/v1/admin/tenants/{project_id}/retention",
            admin.url
        ))
        .method("GET")
        .header("CF-Access-Client-Id", &admin.access_client_id)
        .header("CF-Access-Client-Secret", &admin.access_client_secret)
        .body(Vec::new())?;
    let response = http::Client::new().send(request).await?;
    if response.status() == http::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let status = response.status();
    let body = response.into_body().bytes().await?.to_vec();
    if !status.is_success() {
        anyhow::bail!(
            "signy admin GET tenants/{project_id}/retention answered {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }
    let response: TenantPolicyResponse = serde_json::from_slice(&body)?;
    Ok(Some(TelemetryPolicy {
        revision: response.revision,
        base_retention: response.retention,
        log_retention_override: response.log_retention,
        trace_retention_override: response.trace_retention,
        metric_retention_override: response.metric_retention,
        max_stored_bytes: response
            .max_stored_bytes
            .unwrap_or_else(|| "unlimited".to_string()),
    }))
}

pub async fn revoke_project_access(project_id: &str, fence: u64) -> anyhow::Result<()> {
    SignyAdmin::from_env()?
        .send(
            "POST",
            &format!("project-tenants/{project_id}/access/revoke"),
            serde_json::to_vec(&serde_json::json!({ "fence": fence }))?,
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TenantPolicyResponse;

    #[test]
    fn legacy_policy_without_revision_deserializes_as_zero() {
        let policy: TenantPolicyResponse = serde_json::from_str(
            r#"{"tenant":"project","retention":"30d","max_stored_bytes":"512MiB"}"#,
        )
        .unwrap();

        assert_eq!(policy.revision, 0);
        assert_eq!(policy.retention, "30d");
        assert_eq!(policy.max_stored_bytes.as_deref(), Some("512MiB"));
        assert_eq!(policy.log_retention, None);
        assert_eq!(policy.trace_retention, None);
        assert_eq!(policy.metric_retention, None);
    }
}
