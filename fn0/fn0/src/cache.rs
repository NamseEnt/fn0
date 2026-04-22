use crate::Deployment;
use crate::execute::ClientState;
use crate::measure_cpu_time::SystemClock;
use std::sync::Arc;
use wasmtime::Engine;
use wasmtime::component::{Component, Linker};
use wasmtime_wasi_http::p3::bindings::ServicePre;

pub struct Bundle {
    pub service_pre: ServicePre<ClientState<SystemClock>>,
    pub js: Option<String>,
    pub deployment: Deployment,
    pub env_vars: Vec<(String, String)>,
}

#[allow(async_fn_in_trait)]
pub trait BundleCache: Send + Sync + 'static {
    async fn get(&self, subdomain: &str) -> Result<Arc<Bundle>, Error>;

    async fn invalidate(&self, subdomain: &str);
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    Storage(anyhow::Error),
    Compile(wasmtime::Error),
    Decode(anyhow::Error),
    SingleflightLeaderFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "bundle not found"),
            Error::Storage(e) => write!(f, "storage error: {e:?}"),
            Error::Compile(e) => write!(f, "wasm compile error: {e:?}"),
            Error::Decode(e) => write!(f, "bundle decode error: {e:?}"),
            Error::SingleflightLeaderFailed => write!(f, "singleflight leader failed"),
        }
    }
}

impl std::error::Error for Error {}

pub fn build_service_pre(
    engine: &Engine,
    linker: &Linker<ClientState<SystemClock>>,
    cwasm: &[u8],
) -> Result<ServicePre<ClientState<SystemClock>>, wasmtime::Error> {
    let component = unsafe { Component::deserialize(engine, cwasm) }?;
    let instance_pre = linker.instantiate_pre(&component)?;
    ServicePre::new(instance_pre)
}
