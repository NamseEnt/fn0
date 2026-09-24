use doc_db::DodbConnection;
use dodb_server::{
    DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
};
use fn0_shared_schema::{
    DbRequest, WorkerCertManifestDoc, WorkerCertManifestDocGet, WorkerCertManifestDocPut,
    WorkerManifestDoc, WorkerManifestDocGet, WorkerManifestDocPut,
};
use rcgen::generate_simple_self_signed;
use std::collections::HashMap;
use std::sync::Arc;

pub struct LocalDodb {
    pub connection: DodbConnection,
    server: Arc<DodbServer<LocalTenantService>>,
    task: tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
    _directory: tempfile::TempDir,
}

impl LocalDodb {
    pub async fn start() -> Self {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let certificate = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let certificate_der = certificate.cert.der().to_vec();
        let private_key = certificate.signing_key.serialize_der();
        let directory = tempfile::tempdir().unwrap();
        let service = Arc::new(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_owned(),
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
        let connection = DodbConnection::connect(&doc_db::DodbConfig::new(
            server.local_addr().unwrap(),
            "localhost",
            vec![certificate_der],
        ))
        .await
        .unwrap();
        Self {
            connection,
            server,
            task,
            _directory: directory,
        }
    }

    pub async fn shutdown(self) {
        self.connection.close();
        self.server.shutdown().await;
        self.task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn worker_control_consumers_share_the_dodb_control_tenant() {
    let dodb = LocalDodb::start().await;
    let control_database = doc_db::dodb_with_connection(&dodb.connection, "fn0-control").unwrap();
    let manifest = WorkerManifestDoc {
        manifest_version: 4,
        project_manifests: HashMap::new(),
    };
    WorkerManifestDocPut(manifest.clone())
        .send_with(&control_database)
        .await
        .unwrap();
    assert_eq!(manifest.manifest_version, 4);
    assert_eq!(
        WorkerManifestDocGet {}
            .send_with(&control_database)
            .await
            .unwrap()
            .unwrap()
            .manifest_version,
        manifest.manifest_version
    );

    let cert_manifest = WorkerCertManifestDoc {
        cert_version: 7,
        certs: HashMap::new(),
    };
    WorkerCertManifestDocPut(cert_manifest.clone())
        .send_with(&control_database)
        .await
        .unwrap();
    assert_eq!(cert_manifest.cert_version, 7);
    assert_eq!(
        WorkerCertManifestDocGet {}
            .send_with(&control_database)
            .await
            .unwrap()
            .unwrap()
            .cert_version,
        cert_manifest.cert_version
    );

    let directory = crate::websocket_directory::directory_with_database(
        &crate::websocket_directory::WorkerIdentity {
            worker_id: "worker-test".to_string(),
            endpoint: "127.0.0.1:4433".to_string(),
        },
        control_database.clone(),
    )
    .unwrap();
    let owner = crate::websocket_directory::ConnectionOwner {
        project_id: "abcdefgh".to_string(),
        worker_id: "worker-test".to_string(),
        endpoint: "127.0.0.1:4433".to_string(),
    };
    directory
        .put_connection("connection-test", &owner)
        .await
        .unwrap();
    assert_eq!(
        directory
            .lookup_connection("connection-test")
            .await
            .unwrap(),
        Some(owner)
    );
    dodb.shutdown().await;
}
