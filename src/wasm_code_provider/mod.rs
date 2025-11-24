pub mod fs;
pub mod s3;

use crate::execute::ClientState;
use std::future::Future;
use wasmtime::{Engine, component::Linker};
use wasmtime_wasi_http::bindings::ProxyPre;

pub type Result<T> = std::result::Result<T, Error>;
pub trait WasmCodeProvider: Clone + Send + Sync + 'static {
    fn get_proxy_pre(
        &self,
        id: &str,
        engine: &Engine,
        linker: &Linker<ClientState>,
    ) -> impl Future<Output = Result<ProxyPre<ClientState>>> + Send;
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    IoError { error: std::io::Error },
    ProviderError(anyhow::Error),
    WasmTime(wasmtime::Error),
}
impl From<wasmtime::Error> for Error {
    fn from(value: wasmtime::Error) -> Self {
        Self::WasmTime(value)
    }
}
