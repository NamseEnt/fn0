use doc_db_protocol::{DocDbError, DocDbOperation, DocDbRequest, DocDbResponse, DocDbResult};
use fn0::{DocDbService, DocDbServiceFuture};

pub struct DodbDocDbService {
    connection: doc_db::DodbConnection,
}

impl DodbDocDbService {
    pub fn new(connection: doc_db::DodbConnection) -> Self {
        Self { connection }
    }
}

impl DocDbService for DodbDocDbService {
    fn execute<'a>(&'a self, project_id: &'a str, request: DocDbRequest) -> DocDbServiceFuture<'a> {
        if let DocDbOperation::AdminPurgeProject { project_id: target } = &request.operation {
            let connection = self.connection.clone();
            let caller = project_id.to_string();
            let target = target.clone();
            return Box::pin(async move {
                if caller != "fn0-control" {
                    return Ok(DocDbResponse::error(DocDbError::Forbidden {
                        message: "only fn0-control can purge a project tenant".to_string(),
                    }));
                }
                if target == "fn0-control" {
                    return Ok(DocDbResponse::error(DocDbError::InvalidRequest {
                        message: "fn0-control tenant cannot be purged".to_string(),
                    }));
                }
                if let Err(error) = doc_db::dodb_tenant_id(&target) {
                    return Ok(DocDbResponse::error(DocDbError::InvalidRequest {
                        message: error.to_string(),
                    }));
                }
                match purge_project_tenant(&connection, &target).await {
                    Ok(deleted_rows) => Ok(DocDbResponse::new(DocDbResult::AdminPurgeProject {
                        deleted_rows,
                    })),
                    Err(error) => Err(error.to_string()),
                }
            });
        }
        let database = match doc_db::dodb_with_connection(&self.connection, project_id) {
            Ok(database) => database,
            Err(error) => return Box::pin(async move { Err(error.to_string()) }),
        };
        Box::pin(async move {
            database
                .execute_semantic(request)
                .await
                .map_err(|error| error.to_string())
        })
    }
}

async fn purge_project_tenant(
    connection: &doc_db::DodbConnection,
    project_id: &str,
) -> anyhow::Result<u64> {
    const PURGE_PAGE_SIZE: usize = 256;
    let limits = dodb_protocol::ProtocolLimits::default();
    let page_limit = PURGE_PAGE_SIZE
        .min(limits.max_mutations)
        .min(limits.max_scan_limit);
    let database = doc_db::dodb_with_connection(connection, project_id)?;
    let mut deleted_rows = 0_u64;
    loop {
        let page = database.scan(None, page_limit).await?;
        if page.is_empty() {
            let final_scan = database.scan(None, 1).await?;
            if final_scan.is_empty() {
                return Ok(deleted_rows);
            }
            continue;
        }
        for (pk, sk, _) in page {
            database.delete(&pk, &sk).await?;
            deleted_rows = deleted_rows
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("purge deleted row count overflow"))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_db_protocol::{DocDbOperation, DocDbRequest};
    use dodb_server::{
        DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
    };
    use rcgen::generate_simple_self_signed;
    use std::sync::Arc;

    async fn test_service() -> (
        DodbDocDbService,
        doc_db::DodbConnection,
        Arc<DodbServer<LocalTenantService>>,
        tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
        tempfile::TempDir,
    ) {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let certificate = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let certificate_der = certificate.cert.der().to_vec();
        let private_key = certificate.signing_key.serialize_der();
        let directory = tempfile::tempdir().unwrap();
        let data_path = directory.path().to_owned();
        let service = Arc::new(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir: data_path,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let server = Arc::new(
            DodbServer::bind(
                service,
                DodbServerConfig {
                    listen_addr: "127.0.0.1:0".parse().unwrap(),
                    tls: ServerTlsConfig::from_der(vec![certificate_der.clone()], private_key)
                        .unwrap(),
                    protocol_limits: Default::default(),
                    max_connections: 8,
                    max_concurrent_streams: 64,
                    max_concurrent_requests: 64,
                },
            )
            .unwrap(),
        );
        let task_server = server.clone();
        let task = tokio::spawn(async move { task_server.run().await });
        let connection = doc_db::DodbConnection::connect(&doc_db::DodbConfig::new(
            server.local_addr().unwrap(),
            "localhost",
            vec![certificate_der],
        ))
        .await
        .unwrap();
        (
            DodbDocDbService::new(connection.clone()),
            connection,
            server,
            task,
            directory,
        )
    }

    async fn stop_test_service(
        connection: &doc_db::DodbConnection,
        server: Arc<DodbServer<LocalTenantService>>,
        task: tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
    ) {
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    async fn purge(service: &DodbDocDbService, caller: &str, target: &str) -> DocDbResponse {
        service
            .execute(
                caller,
                DocDbRequest::new(DocDbOperation::AdminPurgeProject {
                    project_id: target.to_string(),
                }),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn purge_authorization_and_target_validation_are_enforced() {
        let (service, connection, server, task, _directory) = test_service().await;
        for (caller, target) in [
            ("abcdefgh", "abcdefgh"),
            ("abcdefgh", "ijklmnop"),
            ("abcdefgh", "fn0-control"),
        ] {
            assert!(matches!(
                purge(&service, caller, target).await.result,
                DocDbResult::Error {
                    error: DocDbError::Forbidden { .. }
                }
            ));
        }
        for target in ["fn0-control", "local", "not-valid"] {
            assert!(matches!(
                purge(&service, "fn0-control", target).await.result,
                DocDbResult::Error {
                    error: DocDbError::InvalidRequest { .. }
                }
            ));
        }
        stop_test_service(&connection, server, task).await;
    }

    #[tokio::test]
    async fn control_purge_removes_all_rows_and_is_idempotent() {
        let (service, connection, server, task, _directory) = test_service().await;
        let project_db = doc_db::dodb_with_connection(&connection, "abcdefgh").unwrap();
        let control_db = doc_db::dodb_with_connection(&connection, "fn0-control").unwrap();
        for row_index in 0..600 {
            project_db
                .put(
                    &format!("partition-{}", row_index % 3),
                    &format!("row-{row_index:04}"),
                    b"payload",
                )
                .await
                .unwrap();
        }
        control_db
            .put("control", "preserve", b"control")
            .await
            .unwrap();
        project_db.delete("partition-0", "row-0000").await.unwrap();

        assert!(matches!(
            purge(&service, "fn0-control", "abcdefgh").await.result,
            DocDbResult::AdminPurgeProject { deleted_rows: 599 }
        ));
        assert!(project_db.scan(None, 1).await.unwrap().is_empty());
        assert!(matches!(
            purge(&service, "fn0-control", "abcdefgh").await.result,
            DocDbResult::AdminPurgeProject { deleted_rows: 0 }
        ));
        assert_eq!(control_db.scan(None, 2).await.unwrap().len(), 1);
        stop_test_service(&connection, server, task).await;
    }

    #[tokio::test]
    async fn empty_project_purge_succeeds() {
        let (service, connection, server, task, _directory) = test_service().await;
        assert!(matches!(
            purge(&service, "fn0-control", "abcdefgh").await.result,
            DocDbResult::AdminPurgeProject { deleted_rows: 0 }
        ));
        stop_test_service(&connection, server, task).await;
    }

    #[tokio::test]
    async fn guest_semantic_documents_are_isolated_by_project() {
        let (service, connection, server, task, _directory) = test_service().await;
        let put_response = service
            .execute(
                "abcdefgh",
                DocDbRequest::new(DocDbOperation::Put {
                    key: doc_db_protocol::DocDbKey::new("partition", "key"),
                    data: b"project-a".to_vec(),
                }),
            )
            .await
            .unwrap();
        assert!(matches!(put_response.result, DocDbResult::Put));

        let other_project_response = service
            .execute(
                "ijklmnop",
                DocDbRequest::new(DocDbOperation::Get {
                    key: doc_db_protocol::DocDbKey::new("partition", "key"),
                }),
            )
            .await
            .unwrap();
        assert!(matches!(
            other_project_response.result,
            DocDbResult::Get { data: None }
        ));
        stop_test_service(&connection, server, task).await;
    }
}
