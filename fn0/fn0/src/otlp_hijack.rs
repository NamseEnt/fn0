use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::header::{AUTHORIZATION, HOST, HeaderName, HeaderValue};
use hyper::http::uri::Scheme;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const PROJECT_ID_HEADER: HeaderName = HeaderName::from_static("x-fn0-project-id");

#[derive(Clone, Debug)]
pub struct OtlpHijack {
    pub placeholder_host: String,
    pub target_scheme: Scheme,
    pub target_host: String,
    pub target_path_prefix: String,
    pub auth: Option<String>,
}

impl OtlpHijack {
    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn rewrite(
        &self,
        req: &mut hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
        caller_project_id: &str,
    ) -> Result<(), ErrorCode> {
        let original_path = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());
        let new_path = format!("{}{}", self.target_path_prefix, original_path);

        let new_uri = hyper::Uri::builder()
            .scheme(self.target_scheme.clone())
            .authority(self.target_host.as_str())
            .path_and_query(new_path.as_str())
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;

        *req.uri_mut() = new_uri;

        let headers = req.headers_mut();
        headers.remove(HOST);
        headers.insert(
            HOST,
            HeaderValue::from_str(&self.target_host)
                .map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
        );

        if let Some(auth) = &self.auth {
            let auth_value = format!("Basic {auth}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value).map_err(|_| ErrorCode::HttpRequestDenied)?,
            );
        } else {
            headers.remove(AUTHORIZATION);
        }

        headers.insert(
            PROJECT_ID_HEADER,
            HeaderValue::from_str(caller_project_id)
                .map_err(|_| ErrorCode::HttpRequestDenied)?,
        );

        Ok(())
    }
}
