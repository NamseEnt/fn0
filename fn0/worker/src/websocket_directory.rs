use async_trait::async_trait;
use dashmap::DashMap;
use doc_db::TrxResult;
use fn0_shared_schema::{
    DbRequest, WebSocketConnectionDoc, WebSocketConnectionDocGet, WebSocketConnectionDocPut,
};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionOwner {
    pub project_id: String,
    pub worker_id: String,
    pub endpoint: String,
}

#[derive(Clone, Debug)]
pub struct WorkerIdentity {
    pub worker_id: String,
    pub endpoint: String,
}

#[async_trait]
pub trait ConnectionDirectory: Send + Sync {
    async fn put_connection(
        &self,
        connection_id: &str,
        owner: &ConnectionOwner,
    ) -> anyhow::Result<()>;
    async fn lookup_connection(
        &self,
        connection_id: &str,
    ) -> anyhow::Result<Option<ConnectionOwner>>;
    async fn delete_connection(&self, connection_id: &str, worker_id: &str) -> anyhow::Result<()>;
}

pub fn directory_from_env(
    identity: &WorkerIdentity,
) -> anyhow::Result<Arc<dyn ConnectionDirectory>> {
    if identity.endpoint.is_empty() {
        return Ok(Arc::new(MemoryDirectory::default()));
    }
    let group_token = std::env::var("TURSO_GROUP_TOKEN")
        .map_err(|_| anyhow::anyhow!("TURSO_GROUP_TOKEN not set"))?;
    let host_suffix = std::env::var("TURSO_DB_HOST_SUFFIX")
        .map_err(|_| anyhow::anyhow!("TURSO_DB_HOST_SUFFIX not set"))?;
    let control_project_id = std::env::var("FN0_CONTROL_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("FN0_CONTROL_PROJECT_ID not set"))?;
    let url = format!("https://{control_project_id}{host_suffix}");
    Ok(Arc::new(TursoDirectory {
        database: doc_db::turso_with_config(url, group_token),
    }))
}

pub fn worker_identity_from_env() -> WorkerIdentity {
    WorkerIdentity {
        worker_id: random_identifier("worker"),
        endpoint: std::env::var("FN0_WEBSOCKET_QUIC_ENDPOINT").unwrap_or_default(),
    }
}

fn random_identifier(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        crate::websocket::WebSocketService::connection_id()
    )
}

struct TursoDirectory {
    database: doc_db::Database,
}

#[async_trait]
impl ConnectionDirectory for TursoDirectory {
    async fn put_connection(
        &self,
        connection_id: &str,
        owner: &ConnectionOwner,
    ) -> anyhow::Result<()> {
        WebSocketConnectionDocPut(WebSocketConnectionDoc {
            connection_id: connection_id.to_string(),
            project_id: owner.project_id.clone(),
            worker_id: owner.worker_id.clone(),
            endpoint: owner.endpoint.clone(),
        })
        .send_with(&self.database)
        .await
    }

    async fn lookup_connection(
        &self,
        connection_id: &str,
    ) -> anyhow::Result<Option<ConnectionOwner>> {
        Ok((WebSocketConnectionDocGet {
            connection_id: connection_id.to_string(),
        })
        .send_with(&self.database)
        .await?
        .map(|document| ConnectionOwner {
            project_id: document.project_id,
            worker_id: document.worker_id,
            endpoint: document.endpoint,
        }))
    }

    async fn delete_connection(&self, connection_id: &str, worker_id: &str) -> anyhow::Result<()> {
        let connection_id = connection_id.to_string();
        let worker_id = worker_id.to_string();
        let result = self
            .database
            .trx(|transaction| {
                let connection_id = connection_id.clone();
                let worker_id = worker_id.clone();
                async move {
                    if let Some(connection) = transaction
                        .get(WebSocketConnectionDocGet { connection_id })
                        .await?
                        && connection.worker_id == worker_id
                    {
                        connection.delete();
                    }
                    transaction.commit::<_, std::convert::Infallible>(())
                }
            })
            .await;
        match result {
            TrxResult::Committed(()) => Ok(()),
            TrxResult::Cancelled(cancel) => match cancel {},
            TrxResult::Conflict(error) => {
                anyhow::bail!("websocket directory delete conflict: {error:?}")
            }
            TrxResult::Err(error) => Err(error),
        }
    }
}

#[derive(Default)]
pub(crate) struct MemoryDirectory {
    connections: DashMap<String, ConnectionOwner>,
}

#[async_trait]
impl ConnectionDirectory for MemoryDirectory {
    async fn put_connection(
        &self,
        connection_id: &str,
        owner: &ConnectionOwner,
    ) -> anyhow::Result<()> {
        self.connections
            .insert(connection_id.to_string(), owner.clone());
        Ok(())
    }

    async fn lookup_connection(
        &self,
        connection_id: &str,
    ) -> anyhow::Result<Option<ConnectionOwner>> {
        Ok(self
            .connections
            .get(connection_id)
            .map(|entry| entry.value().clone()))
    }

    async fn delete_connection(&self, connection_id: &str, worker_id: &str) -> anyhow::Result<()> {
        self.connections
            .remove_if(connection_id, |_, owner| owner.worker_id == worker_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_delete_checks_worker_id() {
        let directory = MemoryDirectory::default();
        let owner = ConnectionOwner {
            project_id: "project".to_string(),
            worker_id: "new".to_string(),
            endpoint: "127.0.0.1:4433".to_string(),
        };
        directory
            .put_connection("connection", &owner)
            .await
            .expect("put connection");
        directory
            .delete_connection("connection", "old")
            .await
            .expect("conditional delete");
        assert_eq!(
            directory
                .lookup_connection("connection")
                .await
                .expect("lookup"),
            Some(owner)
        );
    }

    #[tokio::test]
    async fn turso_directory_round_trip_uses_document_store() {
        let directory = TursoDirectory {
            database: doc_db::memory(),
        };
        let owner = ConnectionOwner {
            project_id: "project".to_string(),
            worker_id: "worker".to_string(),
            endpoint: "127.0.0.1:4433".to_string(),
        };
        directory
            .put_connection("connection", &owner)
            .await
            .expect("put connection");
        assert_eq!(
            directory
                .lookup_connection("connection")
                .await
                .expect("lookup"),
            Some(owner)
        );
        directory
            .delete_connection("connection", "worker")
            .await
            .expect("delete connection");
        assert_eq!(
            directory
                .lookup_connection("connection")
                .await
                .expect("lookup"),
            None
        );
    }
}
