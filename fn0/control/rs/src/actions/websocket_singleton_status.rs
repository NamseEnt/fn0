use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const LEASE_SECONDS: i64 = 60;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub singleton_id: String,
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
    let disconnected = matches!(req.body.status, Status::Disconnected);
    let lease_expires_at = now() + chrono::Duration::seconds(LEASE_SECONDS);
    let accepted = match update_runtime_status(
        &db,
        &req.body.project_id,
        &req.body.singleton_id,
        &req.body.connection_id,
        disconnected,
        lease_expires_at,
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
        let manifest = match (WorkerManifestDocGet {}).send_with(&db).await {
            Ok(Some(manifest)) => manifest,
            Ok(None) => return Output::Ignored,
            Err(_) => return Output::Error,
        };
        let Some(entry) = manifest.project_manifests.get(&req.body.project_id) else {
            return Output::Ignored;
        };
        if let Err(error) =
            crate::enqueue::deployment_activation(crate::queue_task::deployment_activation::Input {
                project_id: req.body.project_id.clone(),
                code_version: entry.code_version,
            })
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
    connection_id: &str,
    disconnected: bool,
    lease_expires_at: DateTime,
) -> anyhow::Result<bool> {
    let project_id = project_id.to_string();
    let singleton_id = singleton_id.to_string();
    let connection_id = connection_id.to_string();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let singleton_id = singleton_id.clone();
            let connection_id = connection_id.clone();
            async move {
                let Some(mut runtime) = trx
                    .get(WebSocketSingletonRuntimeDocGet {
                        project_id: project_id.as_str(),
                        singleton_id: singleton_id.as_str(),
                    })
                    .await?
                else {
                    return trx.commit::<_, ()>(disconnected);
                };
                if runtime.connection_id != connection_id {
                    return trx.commit::<_, ()>(false);
                }
                if disconnected {
                    runtime.delete();
                } else {
                    runtime.lease_expires_at = lease_expires_at;
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
        WebSocketSingletonRuntimeDocPut,
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
                connection_id: "replacement".to_string(),
                lease_expires_at,
            })
            .send_with(&db)
            .await
            .unwrap();
            let accepted =
                update_runtime_status(&db, "project", "feed", "old", true, lease_expires_at)
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
    fn disconnect_before_runtime_write_requests_reconnect() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let accepted = update_runtime_status(
                &db,
                "project",
                "feed",
                "connection",
                true,
                now() + chrono::Duration::seconds(60),
            )
            .await
            .unwrap();
            assert!(accepted);
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
                "connection",
                false,
                now() + chrono::Duration::seconds(60),
            )
            .await
            .unwrap();
            assert!(!accepted);
        });
    }
}
