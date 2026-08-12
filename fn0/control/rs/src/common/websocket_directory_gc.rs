use fn0_shared_schema::{
    DbRequest, WebSocketConnectionDoc, WebSocketConnectionDocGet, WebSocketConnectionDocQuery,
    WebSocketDirectoryGcCursorDoc, WebSocketDirectoryGcCursorDocGet,
    WebSocketDirectoryGcCursorDocPut,
};
use forte_sdk::*;
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const PAGE_LIMIT: usize = 256;
const RECONCILE_PATH: &str = "/__fn0_websocket/reconcile";

#[derive(Serialize)]
struct ReconcileRequest<'a> {
    worker_id: &'a str,
    connection_ids: Vec<&'a str>,
}

#[derive(Deserialize)]
struct ReconcileResponse {
    worker_id: String,
    present_connection_ids: Vec<String>,
}

pub struct GcStats {
    pub scanned_connections: u64,
    pub deleted_connections: u64,
    pub unreachable_workers: u64,
}

pub async fn run_gc() -> anyhow::Result<GcStats> {
    let database = doc_db::turso();
    let cursor = (WebSocketDirectoryGcCursorDocGet {})
        .send_with(&database)
        .await?
        .and_then(|cursor| cursor.after_connection_id);
    let connections: Vec<WebSocketConnectionDoc> = (WebSocketConnectionDocQuery {
        connection_id: cursor,
        limit: Some(PAGE_LIMIT),
    })
    .send_with(&database)
    .await?;
    if connections.is_empty() {
        WebSocketDirectoryGcCursorDocPut(WebSocketDirectoryGcCursorDoc {
            after_connection_id: None,
        })
        .send_with(&database)
        .await?;
        return Ok(GcStats {
            scanned_connections: 0,
            deleted_connections: 0,
            unreachable_workers: 0,
        });
    }
    let last_connection_id = connections
        .last()
        .map(|connection| connection.connection_id.clone());
    WebSocketDirectoryGcCursorDocPut(WebSocketDirectoryGcCursorDoc {
        after_connection_id: last_connection_id,
    })
    .send_with(&database)
    .await?;

    let mut groups: HashMap<(String, String), Vec<WebSocketConnectionDoc>> = HashMap::new();
    for connection in connections {
        groups
            .entry((connection.worker_id.clone(), connection.endpoint.clone()))
            .or_default()
            .push(connection);
    }

    let bearer = std::env::var("FN0_WEBSOCKET_QUIC_BEARER")
        .map_err(|_| anyhow::anyhow!("FN0_WEBSOCKET_QUIC_BEARER not set"))?;
    let mut probes = FuturesUnordered::new();
    for ((worker_id, endpoint), connections) in groups {
        let bearer = bearer.clone();
        probes.push(async move {
            let response = probe_worker(&bearer, &worker_id, &endpoint, &connections).await;
            (worker_id, connections, response)
        });
    }

    let mut scanned_connections = 0_u64;
    let mut deleted_connections = 0_u64;
    let mut unreachable_workers = 0_u64;
    while let Some((worker_id, connections, response)) = probes.next().await {
        scanned_connections += connections.len() as u64;
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                unreachable_workers += 1;
                tracing::warn!(%worker_id, %error, "websocket directory worker probe failed");
                continue;
            }
        };
        let present: HashSet<String> = response.present_connection_ids.into_iter().collect();
        let worker_replaced = response.worker_id != worker_id;
        for connection in connections {
            if !worker_replaced && present.contains(&connection.connection_id) {
                continue;
            }
            if delete_if_owned(&database, &connection.connection_id, &worker_id).await? {
                deleted_connections += 1;
            }
        }
    }
    Ok(GcStats {
        scanned_connections,
        deleted_connections,
        unreachable_workers,
    })
}

async fn probe_worker(
    bearer: &str,
    worker_id: &str,
    endpoint: &str,
    connections: &[WebSocketConnectionDoc],
) -> anyhow::Result<ReconcileResponse> {
    let body = serde_json::to_vec(&ReconcileRequest {
        worker_id,
        connection_ids: connections
            .iter()
            .map(|connection| connection.connection_id.as_str())
            .collect(),
    })?;
    let response = http::Client::new()
        .send(
            http::Request::builder()
                .method("POST")
                .uri(format!("http://{endpoint}{RECONCILE_PATH}"))
                .header("authorization", format!("Bearer {bearer}"))
                .header("content-type", "application/json")
                .body(body)?,
        )
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("reconciliation status {}", response.status());
    }
    Ok(response.into_body().json().await?)
}

async fn delete_if_owned(
    database: &doc_db::Database,
    connection_id: &str,
    worker_id: &str,
) -> anyhow::Result<bool> {
    let connection_id = connection_id.to_string();
    let worker_id = worker_id.to_string();
    let result = database
        .trx(|transaction| {
            let connection_id = connection_id.clone();
            let worker_id = worker_id.clone();
            async move {
                let Some(connection) = transaction
                    .get(WebSocketConnectionDocGet { connection_id })
                    .await?
                else {
                    return transaction.commit::<_, std::convert::Infallible>(false);
                };
                if connection.worker_id != worker_id {
                    return transaction.commit::<_, std::convert::Infallible>(false);
                }
                connection.delete();
                transaction.commit::<_, std::convert::Infallible>(true)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(deleted) => Ok(deleted),
        doc_db::TrxResult::Cancelled(cancel) => match cancel {},
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("websocket directory GC conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::delete_if_owned;
    use fn0_shared_schema::{
        DbRequest, WebSocketConnectionDoc, WebSocketConnectionDocGet, WebSocketConnectionDocPut,
    };

    #[test]
    fn conditional_delete_preserves_replaced_owner() {
        futures::executor::block_on(async {
            let database = doc_db::memory();
            WebSocketConnectionDocPut(WebSocketConnectionDoc {
                connection_id: "connection".to_string(),
                project_id: "project".to_string(),
                worker_id: "current-worker".to_string(),
                endpoint: "127.0.0.1:18445".to_string(),
            })
            .send_with(&database)
            .await
            .expect("put connection");

            assert!(
                !delete_if_owned(&database, "connection", "previous-worker")
                    .await
                    .expect("conditional delete")
            );
            assert!(
                WebSocketConnectionDocGet {
                    connection_id: "connection".to_string(),
                }
                .send_with(&database)
                .await
                .expect("get connection")
                .is_some()
            );

            assert!(
                delete_if_owned(&database, "connection", "current-worker")
                    .await
                    .expect("owned delete")
            );
            assert!(
                WebSocketConnectionDocGet {
                    connection_id: "connection".to_string(),
                }
                .send_with(&database)
                .await
                .expect("get deleted connection")
                .is_none()
            );
        });
    }
}
