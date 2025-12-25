use hyper::Request;
use wasmtime::{Store, component::Component};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};
use wasmtime_wasi_http::{
    WasiHttpCtx, WasiHttpView,
    bindings::{ProxyPre, http::types::Scheme},
    body::HyperOutgoingBody,
};

pub struct Config<'a> {
    pub cwasm: &'a [u8],
}

#[derive(Clone)]
pub struct Fn0One {
    proxy_pre: ProxyPre<ClientState>,
}

impl Fn0One {
    pub async fn new<'a>(config: Config<'a>) -> anyhow::Result<Self> {
        let engine = crate::engine::new_engine()?;
        let linker = crate::engine::new_linker::<ClientState>(&engine)?;

        let component = unsafe { Component::deserialize(&engine, config.cwasm)? };
        let instance_pre = linker.instantiate_pre(&component)?;
        let proxy_pre = ProxyPre::new(instance_pre)?;

        Ok(Self { proxy_pre })
    }

    pub async fn run(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> anyhow::Result<hyper::Response<HyperOutgoingBody>> {
        let mut store = Store::new(
            self.proxy_pre.engine(),
            ClientState {
                table: ResourceTable::new(),
                wasi: WasiCtx::builder().inherit_stdio().build(),
                http: WasiHttpCtx::new(),
            },
        );
        store.set_epoch_deadline(1);

        let proxy = self.proxy_pre.instantiate_async(&mut store).await?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = store.data_mut().new_incoming_request(Scheme::Http, req)?;
        let out = store.data_mut().new_response_outparam(tx)?;

        proxy
            .wasi_http_incoming_handler()
            .call_handle(store, req, out)
            .await?;
        let result = rx.await??;

        Ok(result)
    }
}

struct ClientState {
    wasi: WasiCtx,
    http: WasiHttpCtx,
    table: ResourceTable,
}

impl WasiView for ClientState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl WasiHttpView for ClientState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
