//! Request body for uploads.
//!
//! A body is either bytes the caller already holds or, inside a WASM
//! component, an incoming request body forwarded as-is. Forwarding costs no
//! copy: the guest hands the same `StreamReader` to the outgoing request, so an
//! app can pipe an upload to storage without the object ever sitting in linear
//! memory.

use bytes::Bytes;

pub struct Body(pub(crate) Inner);

pub(crate) enum Inner {
    Bytes(Bytes),
    #[cfg(target_arch = "wasm32")]
    Stream(forte_sdk::http::Body),
}

impl Body {
    /// `None` for a forwarded stream, whose length is unknown until it ends.
    /// R2 accepts such a `PUT` as `Transfer-Encoding: chunked`.
    pub(crate) fn known_length(&self) -> Option<usize> {
        match &self.0 {
            Inner::Bytes(bytes) => Some(bytes.len()),
            #[cfg(target_arch = "wasm32")]
            Inner::Stream(_) => None,
        }
    }

    pub(crate) async fn collect(self) -> Bytes {
        match self.0 {
            Inner::Bytes(bytes) => bytes,
            #[cfg(target_arch = "wasm32")]
            Inner::Stream(body) => body.bytes().await,
        }
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Body(Inner::Bytes(value))
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body(Inner::Bytes(Bytes::from(value)))
    }
}

impl From<&[u8]> for Body {
    fn from(value: &[u8]) -> Self {
        Body(Inner::Bytes(Bytes::copy_from_slice(value)))
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        Body(Inner::Bytes(Bytes::from(value)))
    }
}

impl From<&str> for Body {
    fn from(value: &str) -> Self {
        Body(Inner::Bytes(Bytes::copy_from_slice(value.as_bytes())))
    }
}

#[cfg(target_arch = "wasm32")]
impl From<forte_sdk::http::Body> for Body {
    fn from(value: forte_sdk::http::Body) -> Self {
        Body(Inner::Stream(value))
    }
}
