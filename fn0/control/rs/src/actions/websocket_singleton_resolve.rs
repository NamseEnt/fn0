use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub singleton_id: String,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub enum Output {
    Connected {
        connection_id: String,
        lease_expires_at_millis: i64,
        resolved_at_millis: i64,
    },
    Unavailable,
    Unauthorized,
    Error,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if req
        .headers
        .get("x-fn0-internal-websocket-singleton-resolve")
        .and_then(|value| value.to_str().ok())
        != Some("true")
    {
        return Output::Unauthorized;
    }
    match resolve_singleton(
        &doc_db::database(),
        &req.body.project_id,
        &req.body.singleton_id,
        now(),
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(
                project_id = %req.body.project_id,
                singleton_id = %req.body.singleton_id,
                "websocket singleton resolve failed: {error:#}"
            );
            Output::Error
        }
    }
}

async fn resolve_singleton(
    db: &doc_db::Database,
    project_id: &str,
    singleton_id: &str,
    current_time: DateTime,
) -> anyhow::Result<Output> {
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(db).await? else {
        return Ok(Output::Unavailable);
    };
    let Some(entry) = manifest.project_manifests.get(project_id) else {
        return Ok(Output::Unavailable);
    };
    if entry.static_cache_state != fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE {
        return Ok(Output::Unavailable);
    }
    let Some(runtime) = (WebSocketSingletonRuntimeDocGet {
        project_id,
        singleton_id,
    })
    .send_with(db)
    .await?
    else {
        return Ok(Output::Unavailable);
    };
    if runtime.code_version != entry.code_version
        || runtime.connection_id.is_empty()
        || runtime.state != WebSocketSingletonRuntimeState::Active
        || runtime.lease_expires_at <= current_time
    {
        return Ok(Output::Unavailable);
    }
    Ok(Output::Connected {
        connection_id: runtime.connection_id,
        lease_expires_at_millis: runtime.lease_expires_at.timestamp_millis(),
        resolved_at_millis: current_time.timestamp_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::{Output, resolve_singleton};
    use crate::docs::{
        DbRequest, WebSocketSingletonRuntimeDoc, WebSocketSingletonRuntimeDocPut,
        WebSocketSingletonRuntimeState, WorkerManifestDoc, WorkerManifestDocPut,
        WorkerProjectManifest,
    };
    use forte_sdk::{chrono, now};
    use std::collections::HashMap;

    async fn activate(db: &doc_db::Database, project_id: &str, code_version: u64) {
        activate_projects(db, &[(project_id, code_version)]).await;
    }

    async fn activate_projects(db: &doc_db::Database, projects: &[(&str, u64)]) {
        let mut project_manifests = HashMap::new();
        for (project_id, code_version) in projects {
            project_manifests.insert(
                project_id.to_string(),
                WorkerProjectManifest {
                    code_version: *code_version,
                    domain: format!("{project_id}.example.com"),
                    static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE.to_string(),
                    pending_code_version: None,
                    storage: None,
                },
            );
        }
        WorkerManifestDocPut(WorkerManifestDoc {
            manifest_version: 1,
            project_manifests,
        })
        .send_with(db)
        .await
        .unwrap();
    }

    async fn store_runtime(
        db: &doc_db::Database,
        project_id: &str,
        code_version: u64,
        connection_id: &str,
        lease_seconds: i64,
    ) {
        WebSocketSingletonRuntimeDocPut(WebSocketSingletonRuntimeDoc {
            project_id: project_id.to_string(),
            singleton_id: "feed".to_string(),
            code_version,
            claim_token: "claim".to_string(),
            connection_id: connection_id.to_string(),
            lease_expires_at: now() + chrono::Duration::seconds(lease_seconds),
            state: WebSocketSingletonRuntimeState::Active,
        })
        .send_with(db)
        .await
        .unwrap();
    }

    #[test]
    fn current_unexpired_connection_is_resolved() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            activate(&db, "project", 3).await;
            store_runtime(&db, "project", 3, "v1.connection", 60).await;
            let resolved_at = now();
            let output = resolve_singleton(&db, "project", "feed", resolved_at)
                .await
                .unwrap();
            let Output::Connected {
                connection_id,
                lease_expires_at_millis,
                resolved_at_millis,
            } = output
            else {
                panic!("current connection was not resolved");
            };
            assert_eq!(connection_id, "v1.connection");
            assert!(lease_expires_at_millis > resolved_at_millis);
            assert_eq!(resolved_at_millis, resolved_at.timestamp_millis());
        });
    }

    #[test]
    fn pending_claim_expired_lease_and_old_deployment_are_unavailable() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            activate(&db, "claiming", 3).await;
            store_runtime(&db, "claiming", 3, "", 60).await;
            assert_eq!(
                resolve_singleton(&db, "claiming", "feed", now())
                    .await
                    .unwrap(),
                Output::Unavailable
            );

            activate(&db, "expired", 3).await;
            store_runtime(&db, "expired", 3, "v1.connection", -1).await;
            assert_eq!(
                resolve_singleton(&db, "expired", "feed", now())
                    .await
                    .unwrap(),
                Output::Unavailable
            );

            activate(&db, "redeployed", 4).await;
            store_runtime(&db, "redeployed", 3, "v1.connection", 60).await;
            assert_eq!(
                resolve_singleton(&db, "redeployed", "feed", now())
                    .await
                    .unwrap(),
                Output::Unavailable
            );
        });
    }

    #[test]
    fn another_project_cannot_resolve_the_singleton() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            activate_projects(&db, &[("owner", 3), ("intruder", 3)]).await;
            store_runtime(&db, "owner", 3, "v1.connection", 60).await;
            assert_eq!(
                resolve_singleton(&db, "intruder", "feed", now())
                    .await
                    .unwrap(),
                Output::Unavailable
            );
        });
    }
}
