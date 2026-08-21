use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const LEASE_SECONDS: i64 = 60;
const CLAIM_CONFLICT_RETRIES: usize = 4;

#[derive(Clone, Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
    pub code_version: u64,
    pub singleton_id: String,
}

#[derive(Deserialize)]
struct ConnectionOptions {
    url: String,
    headers: Vec<(String, String)>,
    protocols: Vec<String>,
}

struct ConnectRequest {
    project_id: String,
    singleton_id: String,
    url: String,
    route_path: String,
    headers: http::HeaderMap,
    protocols: Vec<String>,
    claim_token: String,
    lease_deadline_millis: i64,
}

#[allow(async_fn_in_trait)]
trait SingletonConnector {
    async fn connect(&self, request: ConnectRequest) -> anyhow::Result<String>;
}

#[allow(async_fn_in_trait)]
trait SingletonInitializer {
    async fn options(
        &self,
        input: &Input,
        declaration: &WebSocketSingletonDeclaration,
    ) -> anyhow::Result<ConnectionOptions>;
}

struct WorkerConnector;
struct ProjectInitializer;

impl SingletonConnector for WorkerConnector {
    async fn connect(&self, request: ConnectRequest) -> anyhow::Result<String> {
        Ok(websocket::connect_singleton(
            &request.project_id,
            &request.singleton_id,
            request.url,
            &request.route_path,
            &request.headers,
            &request.protocols,
            &request.claim_token,
            request.lease_deadline_millis,
        )
        .await?
        .to_string())
    }
}

impl SingletonInitializer for ProjectInitializer {
    async fn options(
        &self,
        input: &Input,
        declaration: &WebSocketSingletonDeclaration,
    ) -> anyhow::Result<ConnectionOptions> {
        invoke_singleton_connect(input, declaration).await
    }
}

enum ClaimResult {
    Acquired {
        claim_token: String,
        lease_expires_at: DateTime,
    },
    Live,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
    reconcile_with(&db, input, &ProjectInitializer, &WorkerConnector).await
}

async fn reconcile_with<Initializer: SingletonInitializer, Connector: SingletonConnector>(
    db: &doc_db::Database,
    input: Input,
    initializer: &Initializer,
    connector: &Connector,
) -> anyhow::Result<()> {
    let Some(manifest) = (WorkerManifestDocGet {}).send_with(db).await? else {
        return Ok(());
    };
    let Some(entry) = manifest.project_manifests.get(&input.project_id) else {
        return Ok(());
    };
    if entry.code_version != input.code_version
        || entry.static_cache_state != fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
    {
        return Ok(());
    }
    let declaration = (WebSocketSingletonConfigDocGet {
        project_id: input.project_id.as_str(),
        code_version: input.code_version,
    })
    .send_with(db)
    .await?
    .and_then(|config| {
        config
            .declarations
            .into_iter()
            .find(|declaration| declaration.singleton_id == input.singleton_id)
    });
    let Some(declaration) = declaration else {
        return Ok(());
    };
    let claim_token = mint_claim_token();
    let claim = acquire_claim(db, &input, claim_token, now()).await?;
    let ClaimResult::Acquired {
        claim_token,
        lease_expires_at,
    } = claim
    else {
        return Ok(());
    };
    let result = connect_claimed(
        db,
        &input,
        declaration,
        claim_token.as_str(),
        lease_expires_at,
        initializer,
        connector,
    )
    .await;
    if result.is_err() {
        release_claim(db, &input, claim_token.as_str()).await?;
    }
    result
}

async fn connect_claimed<Initializer: SingletonInitializer, Connector: SingletonConnector>(
    db: &doc_db::Database,
    input: &Input,
    declaration: WebSocketSingletonDeclaration,
    claim_token: &str,
    lease_expires_at: DateTime,
    initializer: &Initializer,
    connector: &Connector,
) -> anyhow::Result<()> {
    let options = initializer.options(input, &declaration).await?;
    let mut headers = http::HeaderMap::new();
    for (header_name, header_value) in options.headers {
        headers.append(
            http::HeaderName::from_bytes(header_name.as_bytes())?,
            http::HeaderValue::from_str(&header_value)?,
        );
    }
    let connection_id = connector
        .connect(ConnectRequest {
            project_id: input.project_id.clone(),
            singleton_id: input.singleton_id.clone(),
            url: options.url,
            route_path: declaration.route_path,
            headers,
            protocols: options.protocols,
            claim_token: claim_token.to_string(),
            lease_deadline_millis: lease_expires_at.timestamp_millis(),
        })
        .await?;
    let finalized = finalize_claim(
        db,
        input,
        claim_token,
        connection_id.as_str(),
        now() + chrono::Duration::seconds(LEASE_SECONDS),
    )
    .await?;
    if !finalized {
        tracing::warn!(
            project_id = %input.project_id,
            singleton_id = %input.singleton_id,
            %connection_id,
            "websocket singleton claim was replaced before finalization"
        );
    }
    Ok(())
}

async fn invoke_singleton_connect(
    input: &Input,
    declaration: &WebSocketSingletonDeclaration,
) -> anyhow::Result<ConnectionOptions> {
    let placeholder_uri: http::Uri = std::env::var("FN0_CROSS_PROJECT_INVOKE_URL")?.parse()?;
    let scheme = placeholder_uri.scheme_str().unwrap_or("http");
    let placeholder_host = placeholder_uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("cross-project invoke URL has no host"))?;
    let target_url = format!(
        "{scheme}://{}.{placeholder_host}{}",
        input.project_id, declaration.route_path
    );
    let response = http::Client::new()
        .send(
            http::Request::builder()
                .method("POST")
                .uri(target_url)
                .header("x-fn0-internal-websocket-event", "initialize")
                .header("x-fn0-internal-expected-code-version", input.code_version)
                .body(Vec::new())?,
        )
        .await?;
    if !response.status().is_success() {
        anyhow::bail!(
            "singleton initialization returned status {}",
            response.status()
        );
    }
    let body = response.into_body().bytes().await;
    Ok(serde_json::from_slice(&body)?)
}

fn mint_claim_token() -> String {
    let random_bytes = rand::get_random_bytes(16);
    let uuid_bytes: [u8; 16] = random_bytes
        .try_into()
        .expect("random source returned requested byte count");
    Uuid::from_bytes(uuid_bytes).to_string()
}

async fn acquire_claim(
    db: &doc_db::Database,
    input: &Input,
    claim_token: String,
    current_time: DateTime,
) -> anyhow::Result<ClaimResult> {
    for attempt_number in 0..CLAIM_CONFLICT_RETRIES {
        let project_id = input.project_id.clone();
        let singleton_id = input.singleton_id.clone();
        let claim_token = claim_token.clone();
        let code_version = input.code_version;
        let lease_expires_at = current_time + chrono::Duration::seconds(LEASE_SECONDS);
        let result = db
            .trx(|trx| {
                let project_id = project_id.clone();
                let singleton_id = singleton_id.clone();
                let claim_token = claim_token.clone();
                async move {
                    match trx
                        .get(WebSocketSingletonRuntimeDocGet {
                            project_id: project_id.as_str(),
                            singleton_id: singleton_id.as_str(),
                        })
                        .await?
                    {
                        Some(runtime)
                            if runtime.code_version == code_version
                                && runtime.lease_expires_at > current_time =>
                        {
                            return trx.commit::<_, ()>(false);
                        }
                        Some(mut runtime) => {
                            runtime.code_version = code_version;
                            runtime.claim_token = claim_token;
                            runtime.connection_id.clear();
                            runtime.lease_expires_at = lease_expires_at;
                        }
                        None => {
                            trx.create(WebSocketSingletonRuntimeDoc {
                                project_id,
                                singleton_id,
                                code_version,
                                claim_token,
                                connection_id: String::new(),
                                lease_expires_at,
                            })?;
                        }
                    }
                    trx.commit::<_, ()>(true)
                }
            })
            .await;
        match result {
            doc_db::TrxResult::Committed(true) => {
                return Ok(ClaimResult::Acquired {
                    claim_token,
                    lease_expires_at,
                });
            }
            doc_db::TrxResult::Committed(false) => return Ok(ClaimResult::Live),
            doc_db::TrxResult::Cancelled(()) => unreachable!(),
            doc_db::TrxResult::Conflict(error) if attempt_number + 1 < CLAIM_CONFLICT_RETRIES => {
                tracing::debug!(?error, "websocket singleton claim conflict retry");
            }
            doc_db::TrxResult::Conflict(error) => {
                anyhow::bail!("websocket singleton claim conflict: {error:?}")
            }
            doc_db::TrxResult::Err(error) => return Err(error),
        }
    }
    unreachable!()
}

async fn finalize_claim(
    db: &doc_db::Database,
    input: &Input,
    claim_token: &str,
    connection_id: &str,
    lease_expires_at: DateTime,
) -> anyhow::Result<bool> {
    let project_id = input.project_id.clone();
    let singleton_id = input.singleton_id.clone();
    let claim_token = claim_token.to_string();
    let connection_id = connection_id.to_string();
    let code_version = input.code_version;
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
                if runtime.code_version != code_version
                    || runtime.claim_token != claim_token
                    || !runtime.connection_id.is_empty()
                {
                    return trx.commit::<_, ()>(false);
                }
                runtime.connection_id = connection_id;
                runtime.lease_expires_at = lease_expires_at;
                trx.commit::<_, ()>(true)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(finalized) => Ok(finalized),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("websocket singleton finalize conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

async fn release_claim(
    db: &doc_db::Database,
    input: &Input,
    claim_token: &str,
) -> anyhow::Result<()> {
    let project_id = input.project_id.clone();
    let singleton_id = input.singleton_id.clone();
    let claim_token = claim_token.to_string();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let singleton_id = singleton_id.clone();
            let claim_token = claim_token.clone();
            async move {
                if let Some(runtime) = trx
                    .get(WebSocketSingletonRuntimeDocGet {
                        project_id: project_id.as_str(),
                        singleton_id: singleton_id.as_str(),
                    })
                    .await?
                    && runtime.claim_token == claim_token
                    && runtime.connection_id.is_empty()
                {
                    runtime.delete();
                }
                trx.commit::<_, ()>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("websocket singleton claim release conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClaimResult, ConnectRequest, ConnectionOptions, Input, SingletonConnector,
        SingletonInitializer, acquire_claim, finalize_claim, reconcile_with, release_claim,
    };
    use crate::docs::{
        DbRequest, WebSocketSingletonConfigDoc, WebSocketSingletonConfigDocPut,
        WebSocketSingletonDeclaration, WebSocketSingletonRuntimeDocGet, WorkerManifestDoc,
        WorkerManifestDocPut, WorkerProjectManifest,
    };
    use forte_sdk::{chrono, now};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeInitializer;

    impl SingletonInitializer for FakeInitializer {
        async fn options(
            &self,
            _input: &Input,
            _declaration: &WebSocketSingletonDeclaration,
        ) -> anyhow::Result<ConnectionOptions> {
            Ok(ConnectionOptions {
                url: "wss://example.com/feed".to_string(),
                headers: Vec::new(),
                protocols: Vec::new(),
            })
        }
    }

    struct CountingConnector {
        connection_count: Arc<AtomicUsize>,
    }

    impl SingletonConnector for CountingConnector {
        async fn connect(&self, request: ConnectRequest) -> anyhow::Result<String> {
            assert_eq!(request.project_id, "project");
            assert_eq!(request.singleton_id, "feed");
            assert!(!request.claim_token.is_empty());
            let connection_number = self.connection_count.fetch_add(1, Ordering::AcqRel) + 1;
            Ok(format!("connection-{connection_number}"))
        }
    }

    fn input() -> Input {
        Input {
            project_id: "project".to_string(),
            code_version: 42,
            singleton_id: "feed".to_string(),
        }
    }

    #[test]
    fn concurrent_claims_have_one_owner() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current_time = now();
            let singleton_input = input();
            let first = acquire_claim(&db, &singleton_input, "first".to_string(), current_time);
            let second = acquire_claim(&db, &singleton_input, "second".to_string(), current_time);
            let (first_result, second_result) = futures::join!(first, second);
            let acquired_count = [first_result.unwrap(), second_result.unwrap()]
                .into_iter()
                .filter(|result| matches!(result, ClaimResult::Acquired { .. }))
                .count();
            assert_eq!(acquired_count, 1);
        });
    }

    #[test]
    fn duplicate_reconcile_connects_once() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            WorkerManifestDocPut(WorkerManifestDoc {
                manifest_version: 1,
                project_manifests: HashMap::from([(
                    "project".to_string(),
                    WorkerProjectManifest {
                        code_version: 42,
                        domain: "project.example.com".to_string(),
                        static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                            .to_string(),
                        pending_code_version: None,
                        storage: None,
                    },
                )]),
            })
            .send_with(&db)
            .await
            .unwrap();
            WebSocketSingletonConfigDocPut(WebSocketSingletonConfigDoc {
                project_id: "project".to_string(),
                code_version: 42,
                declarations: vec![WebSocketSingletonDeclaration {
                    singleton_id: "feed".to_string(),
                    route_path: "/ws_singleton/feed".to_string(),
                }],
            })
            .send_with(&db)
            .await
            .unwrap();
            let connection_count = Arc::new(AtomicUsize::new(0));
            let connector = CountingConnector {
                connection_count: connection_count.clone(),
            };
            let first = reconcile_with(&db, input(), &FakeInitializer, &connector);
            let second = reconcile_with(&db, input(), &FakeInitializer, &connector);
            let (first_result, second_result) = futures::join!(first, second);
            first_result.unwrap();
            second_result.unwrap();
            assert_eq!(connection_count.load(Ordering::Acquire), 1);
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap()
            .unwrap();
            assert_eq!(runtime.connection_id, "connection-1");
            assert!(!runtime.claim_token.is_empty());
        });
    }

    #[test]
    fn expired_claim_can_be_replaced() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current_time = now();
            let first = acquire_claim(&db, &input(), "first".to_string(), current_time)
                .await
                .unwrap();
            assert!(matches!(first, ClaimResult::Acquired { .. }));
            let second = acquire_claim(
                &db,
                &input(),
                "second".to_string(),
                current_time + chrono::Duration::seconds(61),
            )
            .await
            .unwrap();
            assert!(matches!(second, ClaimResult::Acquired { .. }));
        });
    }

    #[test]
    fn stale_finalize_cannot_replace_new_claim() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current_time = now();
            acquire_claim(&db, &input(), "first".to_string(), current_time)
                .await
                .unwrap();
            acquire_claim(
                &db,
                &input(),
                "second".to_string(),
                current_time + chrono::Duration::seconds(61),
            )
            .await
            .unwrap();
            let finalized = finalize_claim(
                &db,
                &input(),
                "first",
                "old-connection",
                current_time + chrono::Duration::seconds(120),
            )
            .await
            .unwrap();
            assert!(!finalized);
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap()
            .unwrap();
            assert_eq!(runtime.claim_token, "second");
            assert!(runtime.connection_id.is_empty());
        });
    }

    #[test]
    fn releasing_stale_claim_preserves_replacement() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current_time = now();
            acquire_claim(&db, &input(), "first".to_string(), current_time)
                .await
                .unwrap();
            acquire_claim(
                &db,
                &input(),
                "second".to_string(),
                current_time + chrono::Duration::seconds(61),
            )
            .await
            .unwrap();
            release_claim(&db, &input(), "first").await.unwrap();
            let runtime = (WebSocketSingletonRuntimeDocGet {
                project_id: "project",
                singleton_id: "feed",
            })
            .send_with(&db)
            .await
            .unwrap()
            .unwrap();
            assert_eq!(runtime.claim_token, "second");
        });
    }
}
