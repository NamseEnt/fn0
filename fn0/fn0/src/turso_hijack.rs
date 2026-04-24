use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::header::{AUTHORIZATION, HOST, HeaderValue};
use hyper::http::uri::Scheme;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

#[derive(Clone, Debug)]
pub struct TursoHijack {
    pub placeholder_host: String,
    pub target_host_suffix: String,
    pub group_token: String,
}

impl TursoHijack {
    pub fn target_host(&self, subdomain: &str) -> String {
        format!("{}{}", subdomain, self.target_host_suffix)
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn rewrite(
        &self,
        req: &mut hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
        subdomain: &str,
    ) -> Result<(), ErrorCode> {
        let target_host = self.target_host(subdomain);

        let path_and_query = req
            .uri()
            .path_and_query()
            .cloned()
            .unwrap_or_else(|| "/".parse().unwrap());

        let new_uri = hyper::Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(target_host.as_str())
            .path_and_query(path_and_query)
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;

        *req.uri_mut() = new_uri;

        let headers = req.headers_mut();
        headers.remove(HOST);
        headers.insert(
            HOST,
            HeaderValue::from_str(&target_host).map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
        );

        let auth_value = format!("Bearer {}", self.group_token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|_| ErrorCode::HttpRequestDenied)?,
        );

        Ok(())
    }
}
