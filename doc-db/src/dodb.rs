use anyhow::{Result, anyhow, bail};
use base64::Engine;
use bytes::Bytes;
use dodb_client::{ClientError, ClientTlsConfig, DodbClient, DodbConnection as ClientConnection};
use dodb_core::{
    ConditionExpectation, DocumentKey, PrimaryKey, RevisionState, SortKey, TenantId,
    TransactionCondition, TransactionMutation, TransactionRequest,
};
use dodb_protocol::{ApplicationErrorKind, ProtocolLimits};
use std::env;
use std::io::Cursor;
use std::net::SocketAddr;

use crate::{
    ObservedDocument, TransactCondition, TransactConflict, TransactMutation, TransactOutcome,
    TransactRequest,
};

#[derive(Clone, Debug)]
pub struct DodbConfig {
    server_addr: SocketAddr,
    server_name: String,
    bind_addr: SocketAddr,
    root_certificates: Vec<Vec<u8>>,
    protocol_limits: ProtocolLimits,
}

impl DodbConfig {
    pub fn new(
        server_addr: SocketAddr,
        server_name: impl Into<String>,
        root_certificates: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            server_addr,
            server_name: server_name.into(),
            bind_addr: "0.0.0.0:0".parse().expect("valid default bind address"),
            root_certificates,
            protocol_limits: ProtocolLimits::default(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let server_addr = required_env("DODB_ADDR")?.parse::<SocketAddr>()?;
        let server_name = required_env("DODB_SERVER_NAME")?;
        let bind_addr = env::var("DODB_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:0".to_string())
            .parse::<SocketAddr>()?;
        let root_certificates = root_certificates_from_env()?;

        Ok(Self {
            server_addr,
            server_name,
            bind_addr,
            root_certificates,
            protocol_limits: ProtocolLimits::default(),
        })
    }

    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }

    pub fn with_protocol_limits(mut self, protocol_limits: ProtocolLimits) -> Self {
        self.protocol_limits = protocol_limits;
        self
    }
}

#[derive(Clone)]
pub struct DodbConnection {
    inner: ClientConnection,
    protocol_limits: ProtocolLimits,
}

impl DodbConnection {
    pub async fn connect(config: &DodbConfig) -> Result<Self> {
        let tls =
            ClientTlsConfig::from_der(config.root_certificates.clone()).map_err(client_error)?;
        let inner = ClientConnection::connect(
            config.bind_addr,
            config.server_addr,
            &config.server_name,
            tls,
            config.protocol_limits,
        )
        .await
        .map_err(client_error)?;
        Ok(Self {
            inner,
            protocol_limits: config.protocol_limits,
        })
    }

    pub fn close(&self) {
        self.inner.close();
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.inner.remote_addr()
    }

    pub(crate) fn database(&self, project_id: &str) -> Result<DodbDatabase> {
        Ok(DodbDatabase {
            client: self.inner.for_tenant(dodb_tenant_id(project_id)?),
            protocol_limits: self.protocol_limits,
        })
    }
}

/// The tenant outside the normal eight-character base36 project namespace
/// that is reserved for the fn0 control plane.
pub const FN0_CONTROL_DODB_TENANT_ID: u64 = u64::MAX;

/// Converts fn0's existing string project identity to its dodb tenant.
///
/// Normal project IDs are fixed-width lowercase base36 strings. Keeping the
/// fixed width is important: accepting arbitrary base36 text would make
/// aliases such as `1`, `01`, and `00000001` possible. The positional base36
/// representation is injective, so no hash or probabilistic mapping is needed.
pub fn dodb_tenant_id(project_id: &str) -> Result<TenantId> {
    if project_id == "fn0-control" {
        return Ok(TenantId::new(FN0_CONTROL_DODB_TENANT_ID));
    }
    if project_id == "local" {
        bail!("local project has no dodb tenant");
    }
    if project_id.len() != 8
        || !project_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    {
        bail!("invalid project ID {project_id:?}: expected eight lowercase base36 characters");
    }
    let value = u64::from_str_radix(project_id, 36)
        .map_err(|error| anyhow!("invalid project ID {project_id:?}: {error}"))?;
    Ok(TenantId::new(value))
}

/// Compatibility name for callers that describe this boundary as a project
/// tenant lookup rather than a dodb-specific conversion.
pub fn project_tenant_id(project_id: &str) -> Result<TenantId> {
    dodb_tenant_id(project_id)
}

#[derive(Clone)]
pub(crate) struct DodbDatabase {
    client: DodbClient,
    protocol_limits: ProtocolLimits,
}

impl DodbDatabase {
    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match self
            .client
            .get(document_key(pk, sk))
            .await
            .map_err(client_error)?
        {
            RevisionState::Present { value, .. } => Ok(Some(value.into())),
            RevisionState::Missing { .. } => Ok(None),
        }
    }

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        self.client
            .put(document_key(pk, sk), data.to_vec())
            .await
            .map(|_| ())
            .map_err(client_error)
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        self.client
            .delete(document_key(pk, sk))
            .await
            .map(|_| ())
            .map_err(client_error)
    }

    pub(crate) async fn query(
        &self,
        pk: &str,
        after_sk: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        let mut rows = Vec::with_capacity(limit.min(self.protocol_limits.max_query_limit));
        let mut cursor = after_sk.map(str::to_owned);
        while rows.len() < limit {
            let page_limit = (limit - rows.len()).min(self.protocol_limits.max_query_limit);
            let page = self
                .client
                .query(
                    PrimaryKey::new(pk.as_bytes().to_vec()),
                    cursor
                        .as_deref()
                        .map(|sk| SortKey::new(sk.as_bytes().to_vec())),
                    page_limit,
                )
                .await
                .map_err(client_error)?;
            if page.is_empty() {
                break;
            }
            let page_len = page.len();
            let mut last_sort_key = None;
            for document in page {
                let DocumentKey { pk: row_pk, sk } = document.key;
                if row_pk.as_bytes() != pk.as_bytes() {
                    bail!("dodb query returned an unexpected partition key")
                }
                let sort_key = string_from_key_component("sort key", sk.into_bytes())?;
                last_sort_key = Some(sort_key.clone());
                rows.push((sort_key, document.value.into()));
            }
            cursor = last_sort_key;
            if page_len < page_limit {
                break;
            }
        }
        Ok(rows)
    }

    pub(crate) async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        let mut rows = Vec::with_capacity(limit.min(self.protocol_limits.max_scan_limit));
        let mut cursor = after.map(|(pk, sk)| (pk.to_owned(), sk.to_owned()));
        while rows.len() < limit {
            let page_limit = (limit - rows.len()).min(self.protocol_limits.max_scan_limit);
            let page = self
                .client
                .scan(
                    cursor.as_ref().map(|(pk, sk)| document_key(pk, sk)),
                    page_limit,
                )
                .await
                .map_err(client_error)?;
            if page.is_empty() {
                break;
            }
            let page_len = page.len();
            let mut last_key = None;
            for document in page {
                let DocumentKey { pk, sk } = document.key;
                let pk = string_from_key_component("partition key", pk.into_bytes())?;
                let sk = string_from_key_component("sort key", sk.into_bytes())?;
                last_key = Some((pk.clone(), sk.clone()));
                rows.push((pk, sk, document.value.into()));
            }
            cursor = last_key;
            if page_len < page_limit {
                break;
            }
        }
        Ok(rows)
    }

    pub(crate) async fn get_observed(&self, pk: &str, sk: &str) -> Result<ObservedDocument> {
        let state = self
            .client
            .get(document_key(pk, sk))
            .await
            .map_err(client_error)?;
        Ok(observed_document(state))
    }

    pub(crate) async fn transact(&self, request: &TransactRequest) -> Result<TransactOutcome> {
        if request.conditions.is_empty() && request.mutations.is_empty() {
            return Ok(TransactOutcome { conflict: None });
        }

        let dodb_request = TransactionRequest::new(
            request.conditions.iter().map(dodb_condition).collect(),
            request.mutations.iter().map(dodb_mutation).collect(),
        );
        match self.client.transact(dodb_request).await {
            Ok(_) => Ok(TransactOutcome { conflict: None }),
            Err(ClientError::Application(error))
                if error.kind == ApplicationErrorKind::Conflict =>
            {
                let details = error
                    .conflict
                    .as_ref()
                    .ok_or_else(|| anyhow!("dodb conflict response did not include details"))?;
                let condition_index = request
                    .conditions
                    .iter()
                    .position(|condition| condition_matches(condition, details))
                    .ok_or_else(|| {
                        anyhow!("dodb conflict did not match any requested transaction condition")
                    })?;
                Ok(TransactOutcome {
                    conflict: Some(TransactConflict { condition_index }),
                })
            }
            Err(error) => Err(client_error(error)),
        }
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).map_err(|_| anyhow!("{name} must be set"))
}

fn root_certificates_from_env() -> Result<Vec<Vec<u8>>> {
    let pem = match env::var("DODB_ROOT_CERT_PEM") {
        Ok(value) => value.into_bytes(),
        Err(_) => {
            let encoded = required_env("DODB_ROOT_CERT_PEM_BASE64")?;
            base64::engine::general_purpose::STANDARD.decode(encoded)?
        }
    };
    let mut reader = Cursor::new(pem);
    let certificates = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|certificate| certificate.to_vec())
        .collect::<Vec<_>>();
    if certificates.is_empty() {
        bail!("DODB_ROOT_CERT_PEM must contain at least one certificate")
    }
    Ok(certificates)
}

fn client_error(error: ClientError) -> anyhow::Error {
    anyhow!(error.to_string())
}

fn document_key(pk: &str, sk: &str) -> DocumentKey {
    DocumentKey::new(pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

fn string_from_key_component(name: &str, bytes: Vec<u8>) -> Result<String> {
    String::from_utf8(bytes)
        .map_err(|error| anyhow!("dodb returned an invalid UTF-8 {name}: {error}"))
}

fn observed_document(state: RevisionState) -> ObservedDocument {
    match state {
        RevisionState::Present { value, revision } => ObservedDocument::Present {
            data: value.into(),
            revision: crate::DocDbRevision::new(revision.get()),
        },
        RevisionState::Missing { revision } => ObservedDocument::Missing {
            revision: Some(crate::DocDbRevision::new(revision.get())),
        },
    }
}

fn dodb_condition(condition: &TransactCondition) -> TransactionCondition {
    match condition {
        TransactCondition::RevisionEquals {
            pk,
            sk,
            expected_revision,
        } => TransactionCondition::RevisionEquals {
            key: document_key(pk, sk),
            expected_revision: dodb_core::Revision::new(expected_revision.value()),
        },
        TransactCondition::Exists { pk, sk } => TransactionCondition::Exists {
            key: document_key(pk, sk),
        },
        TransactCondition::NotExists { pk, sk } => TransactionCondition::NotExists {
            key: document_key(pk, sk),
        },
    }
}

fn dodb_mutation(mutation: &TransactMutation) -> TransactionMutation {
    match mutation {
        TransactMutation::Put { pk, sk, data } => TransactionMutation::Put {
            key: document_key(pk, sk),
            value: data.clone(),
        },
        TransactMutation::Delete { pk, sk } => TransactionMutation::Delete {
            key: document_key(pk, sk),
        },
    }
}

fn condition_matches(
    condition: &TransactCondition,
    conflict: &dodb_protocol::ConflictDetails,
) -> bool {
    let (key, expectation) = match condition {
        TransactCondition::RevisionEquals {
            pk,
            sk,
            expected_revision,
        } => (
            document_key(pk, sk),
            ConditionExpectation::RevisionEquals(dodb_core::Revision::new(
                expected_revision.value(),
            )),
        ),
        TransactCondition::Exists { pk, sk } => {
            (document_key(pk, sk), ConditionExpectation::Exists)
        }
        TransactCondition::NotExists { pk, sk } => {
            (document_key(pk, sk), ConditionExpectation::NotExists)
        }
    };
    key == conflict.key && expectation == conflict.expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DocDbRevision, TransactCondition, TransactMutation, TransactRequest, dodb_with_connection,
    };
    use dodb_server::{
        DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerError,
        ServerTlsConfig,
    };
    use rcgen::generate_simple_self_signed;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::task::JoinHandle;

    struct TestTls {
        certificate: Vec<u8>,
        private_key: Vec<u8>,
    }

    fn test_tls() -> TestTls {
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        TestTls {
            certificate: certified.cert.der().to_vec(),
            private_key: certified.signing_key.serialize_der(),
        }
    }

    #[test]
    fn normal_project_ids_use_exact_base36_values() {
        let cases = [
            ("00000000", 0),
            ("00000001", 1),
            ("0000000z", 35),
            ("00000010", 36),
            ("zzzzzzzz", 2_821_109_907_455),
        ];
        for (project_id, expected) in cases {
            assert_eq!(dodb_tenant_id(project_id).unwrap(), TenantId::new(expected));
        }
    }

    #[test]
    fn control_uses_a_reserved_tenant_outside_the_project_namespace() {
        assert_eq!(
            dodb_tenant_id("fn0-control").unwrap(),
            TenantId::new(FN0_CONTROL_DODB_TENANT_ID)
        );
        assert!(TenantId::new(2_821_109_907_455) < TenantId::new(u64::MAX));
    }

    #[test]
    fn local_has_no_dodb_tenant() {
        let error = dodb_tenant_id("local").unwrap_err().to_string();
        assert!(error.contains("no dodb tenant"), "{error}");
    }

    #[test]
    fn arbitrary_and_noncanonical_project_ids_are_rejected() {
        for project_id in [
            "1",
            "01",
            "000000001",
            "ABCDEFGH",
            "abc-defg",
            "fn0-foo",
            "",
        ] {
            assert!(
                dodb_tenant_id(project_id).is_err(),
                "{project_id:?} unexpectedly mapped to a dodb tenant"
            );
        }
    }

    #[test]
    fn representative_canonical_ids_are_injective() {
        let project_ids = ["00000000", "00000001", "0000000z", "00000010", "zzzzzzzz"];
        let tenants = project_ids
            .into_iter()
            .map(|project_id| dodb_tenant_id(project_id).unwrap())
            .collect::<Vec<_>>();
        for (index, tenant) in tenants.iter().enumerate() {
            assert!(tenants[index + 1..].iter().all(|other| other != tenant));
        }
    }

    async fn start_server(
        data_dir: PathBuf,
        tls: &TestTls,
    ) -> (
        Arc<DodbServer<LocalTenantService>>,
        JoinHandle<Result<(), ServerError>>,
    ) {
        let service = Arc::new(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let server = Arc::new(
            DodbServer::bind(
                service,
                DodbServerConfig {
                    listen_addr: "127.0.0.1:0".parse().unwrap(),
                    tls: ServerTlsConfig::from_der(
                        vec![tls.certificate.clone()],
                        tls.private_key.clone(),
                    )
                    .unwrap(),
                    protocol_limits: ProtocolLimits::default(),
                    max_connections: 8,
                    max_concurrent_streams: 64,
                    max_concurrent_requests: 64,
                },
            )
            .unwrap(),
        );
        let task_server = Arc::clone(&server);
        let task = tokio::spawn(async move { task_server.run().await });
        (server, task)
    }

    async fn test_connection(
        server: &DodbServer<LocalTenantService>,
        tls: &TestTls,
    ) -> DodbConnection {
        DodbConnection::connect(
            &DodbConfig::new(
                server.local_addr().unwrap(),
                "localhost",
                vec![tls.certificate.clone()],
            )
            .with_protocol_limits(ProtocolLimits::default()),
        )
        .await
        .unwrap()
    }

    async fn stop_server(
        connection: &DodbConnection,
        server: Arc<DodbServer<LocalTenantService>>,
        task: JoinHandle<Result<(), ServerError>>,
    ) {
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn dodb_backend_runs_the_shared_contract_over_real_quic() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let tls = test_tls();
        let (server, task) = start_server(directory.path().to_owned(), &tls).await;
        let connection = test_connection(&server, &tls).await;
        let database = dodb_with_connection(&connection, "00000000").unwrap();
        let new_connection = connection.clone();

        let result = crate::backend_contract::run_revisioned_missing_backend_contract(
            database,
            move || dodb_with_connection(&new_connection, "00000000").unwrap(),
            "dodb",
        )
        .await;

        stop_server(&connection, server, task).await;
        result.unwrap();
    }

    #[tokio::test]
    async fn dodb_backend_rejects_stale_missing_revision_after_delete() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let tls = test_tls();
        let (server, task) = start_server(directory.path().to_owned(), &tls).await;
        let connection = test_connection(&server, &tls).await;
        let database = dodb_with_connection(&connection, "00000001").unwrap();
        let pk = "missing-revision";
        let sk = "key";

        let initial = database.get_observed(pk, sk).await.unwrap();
        assert!(matches!(
            initial,
            ObservedDocument::Missing {
                revision: Some(revision)
            } if revision == DocDbRevision::new(0)
        ));

        database.put(pk, sk, b"present").await.unwrap();
        database.delete(pk, sk).await.unwrap();
        let deleted = database.get_observed(pk, sk).await.unwrap();
        let deleted_revision = match deleted {
            ObservedDocument::Missing {
                revision: Some(revision),
            } => revision,
            _ => panic!("expected an exact missing revision"),
        };
        assert_ne!(deleted_revision, DocDbRevision::new(0));

        let outcome = database
            .transact(&TransactRequest {
                conditions: vec![TransactCondition::RevisionEquals {
                    pk: pk.to_owned(),
                    sk: sk.to_owned(),
                    expected_revision: DocDbRevision::new(0),
                }],
                mutations: vec![TransactMutation::Put {
                    pk: pk.to_owned(),
                    sk: sk.to_owned(),
                    data: b"must-not-commit".to_vec(),
                }],
            })
            .await
            .unwrap();
        assert_eq!(outcome.conflict.unwrap().condition_index, 0);
        assert_eq!(database.get(pk, sk).await.unwrap(), None);

        stop_server(&connection, server, task).await;
    }
}
