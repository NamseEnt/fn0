use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const LEASE_SECONDS: i64 = 60;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub singleton_id: String,
    #[serde(default)]
    pub claim_token: String,
    pub connection_id: String,
    pub status: Status,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Heartbeat,
    Disconnected,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    Ignored,
    Unauthorized,
    Error,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if req
        .headers
        .get("x-fn0-internal-websocket-status")
        .and_then(|value| value.to_str().ok())
        != Some("true")
    {
        return Output::Unauthorized;
    }
    let db = doc_db::turso();
    let manifest = match (WorkerManifestDocGet {}).send_with(&db).await {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return Output::Ignored,
        Err(_) => return Output::Error,
    };
    let Some(entry) = manifest.project_manifests.get(&req.body.project_id) else {
        return Output::Ignored;
    };
    let disconnected = matches!(req.body.status, Status::Disconnected);
    let current_time = now();
    let lease_expires_at = current_time + chrono::Duration::seconds(LEASE_SECONDS);
    let accepted = match update_runtime_status(
        &db,
        &req.body.project_id,
        &req.body.singleton_id,
        &req.body.claim_token,
        &req.body.connection_id,
        disconnected,
        current_time,
        lease_expires_at,
        entry.code_version,
    )
    .await
    {
        Ok(accepted) => accepted,
        Err(_) => return Output::Error,
    };
    if !accepted {
        return Output::Ignored;
    }
    if disconnected {
        if let Err(error) = crate::enqueue::websocket_singleton_reconcile(
            crate::queue_task::websocket_singleton_reconcile::Input {
                project_id: req.body.project_id.clone(),
                code_version: entry.code_version,
                singleton_id: req.body.singleton_id.clone(),
            },
        )
        .await
        {
            tracing::error!(%error, "websocket singleton reconnect enqueue failed");
            return Output::Error;
        }
    }
    Output::Ok
}

async fn update_runtime_status(
    db: &doc_db::Database,
    project_id: &str,
    singleton_id: &str,
    claim_token: &str,
    connection_id: &str,
    disconnected: bool,
    current_time: DateTime,
    lease_expires_at: DateTime,
    active_code_version: u64,
) -> anyhow::Result<bool> {
    let project_id = project_id.to_string();
    let singleton_id = singleton_id.to_string();
    let claim_token = claim_token.to_string();
    let connection_id = connection_id.to_string();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let singleton_id = singleton_id.clone();
            let claim_token = claim_token.clone();
            let connection_id = connection_id.clone();
            async move {
                let Some(mut runtime) = trx
                    .get(WebSocketSingletonRuntimeDocGet {
                        project_id: project_id.as_str(),
                        singleton_id: singleton_id.as_str(),
                    })
                    .await?
                else {
                    return trx.commit::<_, ()>(false);
                };
                let legacy_claim = runtime.claim_token.is_empty() && claim_token.is_empty();
                if (!legacy_claim && runtime.claim_token != claim_token)
                    || (!runtime.connection_id.is_empty() && runtime.connection_id != connection_id)
                    || (!disconnected && runtime.connection_id.is_empty())
                    || (!disconnected && runtime.code_version != active_code_version)
                    || (!disconnected && runtime.lease_expires_at <= current_time)
                {
                    return trx.commit::<_, ()>(false);
                }
                if disconnected {
                    runtime.connection_id.clear();
                    runtime.state = WebSocketSingletonRuntimeState::Terminating;
                } else {
                    if lease_expires_at > runtime.lease_expires_at {
                        runtime.lease_expires_at = lease_expires_at;
                    }
                }
                trx.commit::<_, ()>(true)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(accepted) => Ok(accepted),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("websocket singleton runtime conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::update_runtime_status;
    use crate::docs::{
        DbRequest, WebSocketSingletonRuntimeDoc, WebSocketSingletonRuntimeDocGet,
        WebSocketSingletonRuntimeDocPut, WebSocketSingletonRuntimeState,
    };
    use forte_sdk::{chrono, now};

    #[test]
    fn late_disconnect_does_not_delete_replacement_connection() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let lease_expires_at = now() + chrono::Duration::seconds(60);
            WebSocketSingletonRuntimeDocPut(WebSocketSingletonRuntimeDoc {
                project_id: "project".to_string(),
                singleton_id: "feed".to_string(),
                code_version: 2,
                claim_token: "replacement-claim".to_string(),
                connection_id: "replacement".to_string(),
                lease_expires_at,
                state: WebSocketSingletonRuntimeState::Active,
            })
            .send_with(&db)
            .await
            .unwrap();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "old-claim",
                "old",
                true,
                now(),
                lease_expires_at,
                2,
            )
            .await
            .unwrap();
            assert!(!accepted);
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap()
            .unwrap();
            assert_eq!(runtime.connection_id, "replacement");
        });
    }

    #[test]
    fn disconnect_without_claim_is_ignored() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "claim",
                "connection",
                true,
                now(),
                now() + chrono::Duration::seconds(60),
                2,
            )
            .await
            .unwrap();
            assert!(!accepted);
        });
    }

    #[test]
    fn heartbeat_without_runtime_record_is_ignored() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "claim",
                "connection",
                false,
                now(),
                now() + chrono::Duration::seconds(60),
                2,
            )
            .await
            .unwrap();
            assert!(!accepted);
        });
    }

    #[test]
    fn heartbeat_only_extends_an_existing_reservation() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let existing_expiry = now() + chrono::Duration::seconds(120);
            WebSocketSingletonRuntimeDocPut(WebSocketSingletonRuntimeDoc {
                project_id: "project".to_string(),
                singleton_id: "feed".to_string(),
                code_version: 2,
                claim_token: "claim".to_string(),
                connection_id: "connection".to_string(),
                lease_expires_at: existing_expiry,
                state: WebSocketSingletonRuntimeState::Active,
            })
            .send_with(&db)
            .await
            .unwrap();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "claim",
                "connection",
                false,
                now(),
                now() + chrono::Duration::seconds(60),
                2,
            )
            .await
            .unwrap();
            assert!(accepted);
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap()
            .unwrap();
            assert_eq!(runtime.lease_expires_at, existing_expiry);
        });
    }

    #[test]
    fn heartbeat_at_expiry_cannot_extend_the_reservation() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current_time = now();
            WebSocketSingletonRuntimeDocPut(WebSocketSingletonRuntimeDoc {
                project_id: "project".to_string(),
                singleton_id: "feed".to_string(),
                code_version: 2,
                claim_token: "claim".to_string(),
                connection_id: "connection".to_string(),
                lease_expires_at: current_time,
                state: WebSocketSingletonRuntimeState::Active,
            })
            .send_with(&db)
            .await
            .unwrap();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "claim",
                "connection",
                false,
                current_time,
                current_time + chrono::Duration::seconds(60),
                2,
            )
            .await
            .unwrap();
            assert!(!accepted);
        });
    }

    #[test]
    fn disconnect_during_claim_releases_its_claim() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let lease_expires_at = now() + chrono::Duration::seconds(60);
            WebSocketSingletonRuntimeDocPut(WebSocketSingletonRuntimeDoc {
                project_id: "project".to_string(),
                singleton_id: "feed".to_string(),
                code_version: 2,
                claim_token: "claim".to_string(),
                connection_id: String::new(),
                lease_expires_at,
                state: WebSocketSingletonRuntimeState::Preparing,
            })
            .send_with(&db)
            .await
            .unwrap();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "claim",
                "connection",
                true,
                now(),
                lease_expires_at,
                2,
            )
            .await
            .unwrap();
            assert!(accepted);
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap();
            assert!(runtime.unwrap().connection_id.is_empty());
        });
    }
}
