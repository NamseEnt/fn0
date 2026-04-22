use std::fmt;

use wit_bindgen::rt::async_support::StreamReader;

use crate::bindings::wasi::http::client;
use crate::bindings::wasi::http::types as p3;
use crate::bindings::{wit_future, wit_stream};

pub use bytes::Bytes;
pub use http::request::Builder as RequestBuilder;
pub use http::uri::{Authority, PathAndQuery, Scheme, Uri};
pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode};

pub mod body {
    pub use super::{Body, Bytes};
}

pub enum Body {
    Empty,
    Bytes(Vec<u8>),
    Stream(StreamReader<u8>),
}

#[derive(Debug)]
pub enum Error {
    Headers(p3::HeaderError),
    InvalidScheme,
    InvalidAuthority,
    InvalidPathWithQuery,
    InvalidMethod,
    StreamBodyNotSupported,
    Wasi(p3::ErrorCode),
    BuildResponse(http::Error),
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Headers(e) => write!(f, "invalid headers: {e:?}"),
            Error::InvalidScheme => write!(f, "invalid scheme"),
            Error::InvalidAuthority => write!(f, "invalid authority"),
            Error::InvalidPathWithQuery => write!(f, "invalid path-with-query"),
            Error::InvalidMethod => write!(f, "invalid method"),
            Error::StreamBodyNotSupported => write!(f, "outgoing streaming body not yet supported"),
            Error::Wasi(ec) => write!(f, "wasi http error: {ec:?}"),
            Error::BuildResponse(e) => write!(f, "failed to build response: {e}"),
            Error::Json(e) => write!(f, "failed to decode JSON: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::BuildResponse(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Error::BuildResponse(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl Body {
    pub fn empty() -> Self {
        Body::Empty
    }

    pub async fn bytes(self) -> Bytes {
        match self {
            Body::Empty => Bytes::new(),
            Body::Bytes(v) => Bytes::from(v),
            Body::Stream(reader) => Bytes::from(reader.collect().await),
        }
    }

    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        let bytes = self.bytes().await;
        serde_json::from_slice(&bytes).map_err(Error::Json)
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Body::Bytes(v)
    }
}

impl From<&[u8]> for Body {
    fn from(v: &[u8]) -> Self {
        Body::Bytes(v.to_vec())
    }
}

impl From<String> for Body {
    fn from(v: String) -> Self {
        Body::Bytes(v.into_bytes())
    }
}

impl From<&str> for Body {
    fn from(v: &str) -> Self {
        Body::Bytes(v.as_bytes().to_vec())
    }
}

impl From<Bytes> for Body {
    fn from(v: Bytes) -> Self {
        Body::Bytes(v.to_vec())
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Body::Empty
    }
}

#[derive(Default, Clone, Debug)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send<B: Into<Body>>(&self, req: Request<B>) -> Result<Response<Body>> {
        let (parts, body) = req.into_parts();
        let body: Body = body.into();

        let header_entries: Vec<(String, Vec<u8>)> = parts
            .headers
            .iter()
            .map(|(name, value)| (name.as_str().to_string(), value.as_bytes().to_vec()))
            .collect();
        let fields = p3::Fields::from_list(header_entries)
            .await
            .map_err(Error::Headers)?;

        let body_bytes: Option<Vec<u8>> = match body {
            Body::Empty => None,
            Body::Bytes(b) if b.is_empty() => None,
            Body::Bytes(b) => Some(b),
            Body::Stream(_) => return Err(Error::StreamBodyNotSupported),
        };

        let contents_reader = if let Some(bytes) = body_bytes {
            let (mut writer, reader) = wit_stream::new::<u8>();
            crate::runtime::spawn(async move {
                let _leftover = writer.write_all(bytes).await;
                drop(writer);
            });
            Some(reader)
        } else {
            None
        };

        let (trailers_writer, trailers_reader) = wit_future::new::<
            core::result::Result<Option<p3::Trailers>, p3::ErrorCode>,
        >(|| Ok(None));
        crate::runtime::spawn(async move {
            drop(trailers_writer);
        });

        let (wasi_req, _transmit) =
            p3::Request::new(fields, contents_reader, trailers_reader, None).await;

        wasi_req
            .set_method(convert_method(&parts.method))
            .await
            .map_err(|_| Error::InvalidMethod)?;

        if let Some(scheme) = parts.uri.scheme_str() {
            wasi_req
                .set_scheme(Some(convert_scheme(scheme)))
                .await
                .map_err(|_| Error::InvalidScheme)?;
        }
        if let Some(authority) = parts.uri.authority() {
            wasi_req
                .set_authority(Some(authority.as_str().to_string()))
                .await
                .map_err(|_| Error::InvalidAuthority)?;
        }
        if let Some(pq) = parts.uri.path_and_query() {
            wasi_req
                .set_path_with_query(Some(pq.as_str().to_string()))
                .await
                .map_err(|_| Error::InvalidPathWithQuery)?;
        }

        let wasi_resp = client::send(wasi_req).await.map_err(Error::Wasi)?;

        let status = wasi_resp.get_status_code().await;
        let wasi_headers = wasi_resp.get_headers().await;
        let header_list = wasi_headers.copy_all().await;
        drop(wasi_headers);

        let (res_trailers_writer, res_trailers_reader) =
            wit_future::new::<core::result::Result<(), p3::ErrorCode>>(|| Ok(()));
        crate::runtime::spawn(async move {
            drop(res_trailers_writer);
        });
        let (body_stream, _trailers) =
            p3::Response::consume_body(wasi_resp, res_trailers_reader).await;

        let mut builder = Response::builder().status(status);
        for (name, value) in header_list {
            builder = builder.header(name, value);
        }
        builder.body(Body::Stream(body_stream)).map_err(Error::from)
    }
}

fn convert_method(m: &Method) -> p3::Method {
    match *m {
        Method::GET => p3::Method::Get,
        Method::HEAD => p3::Method::Head,
        Method::POST => p3::Method::Post,
        Method::PUT => p3::Method::Put,
        Method::DELETE => p3::Method::Delete,
        Method::CONNECT => p3::Method::Connect,
        Method::OPTIONS => p3::Method::Options,
        Method::TRACE => p3::Method::Trace,
        Method::PATCH => p3::Method::Patch,
        _ => p3::Method::Other(m.as_str().to_string()),
    }
}

fn convert_scheme(s: &str) -> p3::Scheme {
    match s {
        "http" => p3::Scheme::Http,
        "https" => p3::Scheme::Https,
        other => p3::Scheme::Other(other.to_string()),
    }
}
