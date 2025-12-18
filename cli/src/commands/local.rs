use color_eyre::{eyre::eyre, Result};
use fn0::Fn0;
use hyper::{server::conn::http1, Request};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower::ServiceBuilder;

pub async fn execute(port: Option<u16>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    let wasm_file = PathBuf::from("./dist/component.wasm");

    crate::commands::build::execute().await?;

    let config = fn0::Config {
        port,
        wasm_path: Some(wasm_file),
        otlp_endpoint: None,
    };

    println!("\nServer starting...\n");

    let listener = {
        let port = config.port.unwrap_or(3000);
        let increment_on_fail = config.port.is_none();
        open_tcp_listener(port, increment_on_fail)?
    };

    println!(
        "Server on http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    let code_id = config
        .wasm_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| eyre!("Invalid wasm_path"))?
        .to_string();

    let fn0 = std::sync::Arc::new(Fn0::new(config).await?);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let tower_service = ServiceBuilder::new().service(tower::util::service_fn({
            let code_id = code_id.clone();
            let fn0 = fn0.clone();

            move |req: Request<hyper::body::Incoming>| {
                let code_id = code_id.clone();
                let fn0 = fn0.clone();

                async move { fn0.run(code_id, req).await }
            }
        }));

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    TowerToHyperService::new(tower_service),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn open_tcp_listener(mut port: u16, increment_on_fail: bool) -> Result<TcpListener> {
    loop {
        let socket = try_open_tcp_listener(port);
        if !increment_on_fail || socket.is_ok() {
            return socket.map_err(Into::into);
        }
        if port == u16::MAX {
            return Err(eyre!("Failed to open socket"));
        }
        port += 1;
    }
}

fn try_open_tcp_listener(port: u16) -> std::io::Result<TcpListener> {
    let addr: SocketAddr = format!("[::]:{port}").parse().unwrap();
    let domain = Domain::for_address(addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_only_v6(false)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;

    Ok(listener)
}
