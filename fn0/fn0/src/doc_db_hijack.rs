use doc_db_protocol::{
    CodecError, DocDbError, DocDbRequest, DocDbResponse, DocDbResult, DocDbTransactOutcome,
    decode_request,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type DocDbServiceFuture<'a> =
    Pin<Box<dyn Future<Output = Result<DocDbResponse, String>> + Send + 'a>>;

pub trait DocDbService: Send + Sync {
    fn execute<'a>(&'a self, project_id: &'a str, request: DocDbRequest) -> DocDbServiceFuture<'a>;
}

#[derive(Clone)]
pub struct DocDbHijack {
    pub placeholder_host: String,
    service: Arc<dyn DocDbService>,
}

impl DocDbHijack {
    pub fn new(placeholder_host: String, service: Arc<dyn DocDbService>) -> Self {
        Self {
            placeholder_host,
            service,
        }
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}/rpc", self.placeholder_host)
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str()) && uri.path() == "/rpc"
    }

    pub async fn handle(&self, project_id: &str, body: &[u8]) -> DocDbResponse {
        let request = match decode_request(body) {
            Ok(request) => request,
            Err(CodecError::Malformed(message)) => {
                return DocDbResponse::error(DocDbError::InvalidRequest { message });
            }
            Err(CodecError::UnsupportedVersion(version)) => {
                return DocDbResponse::error(DocDbError::UnsupportedVersion { version });
            }
            Err(CodecError::Serialize(message)) => {
                return DocDbResponse::error(DocDbError::InvalidRequest { message });
            }
        };
        match self.service.execute(project_id, request).await {
            Ok(response) => response,
            Err(message) => DocDbResponse::error(DocDbError::Backend { message }),
        }
    }
}

pub(crate) fn response_status(response: &DocDbResponse) -> u16 {
    match &response.result {
        DocDbResult::Error { error } => match error {
            DocDbError::InvalidRequest { .. } => 400,
            DocDbError::Backend { .. } => 500,
            DocDbError::UnsupportedVersion { .. } => 505,
        },
        DocDbResult::Transact {
            outcome: DocDbTransactOutcome::Conflict { .. },
        } => 409,
        _ => 200,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_db_protocol::{DocDbKey, DocDbOperation, DocDbRequest};
    use std::sync::Mutex;

    struct FakeService {
        calls: Mutex<Vec<(String, DocDbRequest)>>,
        response: DocDbResponse,
    }

    impl DocDbService for FakeService {
        fn execute<'a>(
            &'a self,
            project_id: &'a str,
            request: DocDbRequest,
        ) -> DocDbServiceFuture<'a> {
            self.calls
                .lock()
                .unwrap()
                .push((project_id.to_string(), request));
            let response = self.response.clone();
            Box::pin(async move { Ok(response) })
        }
    }

    fn request() -> DocDbRequest {
        DocDbRequest::new(DocDbOperation::Get {
            key: DocDbKey::new("pk", "sk"),
        })
    }

    #[tokio::test]
    async fn matches_only_the_placeholder_host() {
        let service = Arc::new(FakeService {
            calls: Mutex::new(Vec::new()),
            response: DocDbResponse::new(DocDbResult::Put),
        });
        let hijack = DocDbHijack::new("fn0-doc-db.fn0.dev".to_string(), service);
        assert!(hijack.matches(&"http://fn0-doc-db.fn0.dev/rpc".parse().unwrap()));
        assert!(!hijack.matches(&"http://example.com/rpc".parse().unwrap()));
        assert!(!hijack.matches(&"http://fn0-doc-db.fn0.dev/other".parse().unwrap()));
    }

    #[tokio::test]
    async fn forwards_authoritative_project_identity_and_request() {
        let service = Arc::new(FakeService {
            calls: Mutex::new(Vec::new()),
            response: DocDbResponse::new(DocDbResult::Put),
        });
        let hijack = DocDbHijack::new("fn0-doc-db.fn0.dev".to_string(), service.clone());
        let body = doc_db_protocol::encode_request(&request()).unwrap();
        let response = hijack.handle("authoritative-project", &body).await;
        assert_eq!(response.result, DocDbResult::Put);
        let calls = service.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "authoritative-project");
        assert_eq!(calls[0].1, request());
    }

    #[tokio::test]
    async fn rejects_guest_project_selection() {
        let service = Arc::new(FakeService {
            calls: Mutex::new(Vec::new()),
            response: DocDbResponse::new(DocDbResult::Put),
        });
        let hijack = DocDbHijack::new("fn0-doc-db.fn0.dev".to_string(), service.clone());
        let response = hijack
            .handle(
                "authoritative-project",
                br#"{"version":1,"project_id":"other-project","operation":{"kind":"Get","value":{"key":{"pk":"pk","sk":"sk"}}}}"#,
            )
            .await;
        assert!(matches!(
            response.result,
            DocDbResult::Error {
                error: DocDbError::InvalidRequest { .. }
            }
        ));
        assert!(service.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn returns_protocol_errors_without_forwarding() {
        let service = Arc::new(FakeService {
            calls: Mutex::new(Vec::new()),
            response: DocDbResponse::new(DocDbResult::Put),
        });
        let hijack = DocDbHijack::new("fn0-doc-db.fn0.dev".to_string(), service.clone());
        let malformed = hijack.handle("project", b"not-json").await;
        assert!(matches!(
            malformed.result,
            DocDbResult::Error {
                error: DocDbError::InvalidRequest { .. }
            }
        ));
        assert!(service.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn returns_backend_errors_as_protocol_errors() {
        struct FailingService;

        impl DocDbService for FailingService {
            fn execute<'a>(
                &'a self,
                _project_id: &'a str,
                _request: DocDbRequest,
            ) -> DocDbServiceFuture<'a> {
                Box::pin(async { Err("backend unavailable".to_string()) })
            }
        }

        let hijack = DocDbHijack::new("fn0-doc-db.fn0.dev".to_string(), Arc::new(FailingService));
        let body = doc_db_protocol::encode_request(&request()).unwrap();
        let response = hijack.handle("project", &body).await;
        assert_eq!(
            response.result,
            DocDbResult::Error {
                error: DocDbError::Backend {
                    message: "backend unavailable".to_string()
                }
            }
        );
    }

    #[test]
    fn maps_protocol_errors_and_conflicts_to_http_statuses() {
        assert_eq!(
            response_status(&DocDbResponse::error(DocDbError::InvalidRequest {
                message: "bad".to_string()
            })),
            400
        );
        assert_eq!(
            response_status(&DocDbResponse::error(DocDbError::Backend {
                message: "failed".to_string()
            })),
            500
        );
        assert_eq!(
            response_status(&DocDbResponse::new(DocDbResult::Transact {
                outcome: DocDbTransactOutcome::Conflict { condition_index: 2 },
            })),
            409
        );
    }
}
