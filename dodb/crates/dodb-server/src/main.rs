use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use dodb_protocol::ProtocolLimits;
use dodb_server::{
    DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Parser)]
#[command(name = "dodb-server", about = "Run the dodb raw QUIC server")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:18445")]
    listen: SocketAddr,

    #[arg(long)]
    data_dir: PathBuf,

    #[arg(long)]
    tls_cert: PathBuf,

    #[arg(long)]
    tls_key: PathBuf,

    #[arg(long, default_value_t = 1024)]
    max_open_shards: usize,

    #[arg(long, default_value_t = 8)]
    max_connections: usize,

    #[arg(long, default_value_t = 64)]
    max_concurrent_streams: usize,

    #[arg(long, default_value_t = 64)]
    max_concurrent_requests: usize,
}

impl Args {
    fn validate(&self) -> Result<(), std::io::Error> {
        if self.max_open_shards == 0 {
            return Err(std::io::Error::other("max-open-shards must be nonzero"));
        }
        if self.max_connections == 0 {
            return Err(std::io::Error::other("max-connections must be nonzero"));
        }
        if self.max_concurrent_streams == 0 {
            return Err(std::io::Error::other(
                "max-concurrent-streams must be nonzero",
            ));
        }
        if self.max_concurrent_requests == 0 {
            return Err(std::io::Error::other(
                "max-concurrent-requests must be nonzero",
            ));
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let server = Arc::new(build_server(&args)?);
    let listen_addr = server.local_addr()?;
    println!(
        "dodb-server listening on {listen_addr}; data-dir={}",
        args.data_dir.display()
    );
    run_until_shutdown(server).await
}

fn build_server(args: &Args) -> Result<DodbServer<LocalTenantService>, BoxError> {
    args.validate()?;

    let certificate_pem = std::fs::read(&args.tls_cert).map_err(|error| {
        std::io::Error::other(format!(
            "failed to read TLS certificate {}: {error}",
            args.tls_cert.display()
        ))
    })?;
    let private_key_pem = std::fs::read(&args.tls_key).map_err(|error| {
        std::io::Error::other(format!(
            "failed to read TLS private key {}: {error}",
            args.tls_key.display()
        ))
    })?;
    let tls = ServerTlsConfig::from_pem(&certificate_pem, &private_key_pem).map_err(|error| {
        std::io::Error::other(format!(
            "failed to parse TLS certificate {} and private key {}: {error}",
            args.tls_cert.display(),
            args.tls_key.display()
        ))
    })?;
    let service = Arc::new(LocalTenantService::new(LocalTenantServiceConfig {
        data_dir: args.data_dir.clone(),
        max_open_shards: args.max_open_shards,
        ..LocalTenantServiceConfig::default()
    })?);

    Ok(DodbServer::bind(
        service,
        DodbServerConfig {
            listen_addr: args.listen,
            tls,
            protocol_limits: ProtocolLimits::default(),
            max_connections: args.max_connections,
            max_concurrent_streams: args.max_concurrent_streams,
            max_concurrent_requests: args.max_concurrent_requests,
        },
    )?)
}

enum ServerEvent {
    Signal(std::io::Result<()>),
    Task(Result<Result<(), dodb_server::ServerError>, tokio::task::JoinError>),
}

async fn run_until_shutdown(server: Arc<DodbServer<LocalTenantService>>) -> Result<(), BoxError> {
    let mut server_task = tokio::spawn({
        let server = Arc::clone(&server);
        async move { server.run().await }
    });

    let event = tokio::select! {
        biased;
        task_result = &mut server_task => ServerEvent::Task(task_result),
        signal_result = shutdown_signal() => ServerEvent::Signal(signal_result),
    };

    match event {
        ServerEvent::Signal(Ok(())) => {
            server.shutdown().await;
            match server_task.await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(server_error)) => Err(Box::new(server_error)),
                Err(join_error) => Err(Box::new(join_error)),
            }
        }
        ServerEvent::Signal(Err(signal_error)) => {
            server.shutdown().await;
            let _ = server_task.await;
            Err(Box::new(signal_error))
        }
        ServerEvent::Task(task_result) => {
            server.shutdown().await;
            match task_result {
                Ok(Err(server_error)) => Err(Box::new(server_error)),
                Ok(Ok(())) => Err(std::io::Error::other(
                    "dodb-server stopped unexpectedly without a shutdown signal",
                )
                .into()),
                Err(join_error) => Err(Box::new(join_error)),
            }
        }
    }
}

#[cfg(unix)]
async fn shutdown_signal() -> std::io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = terminate.recv() => {}
        _ = interrupt.recv() => {}
    }

    Ok(())
}

#[cfg(not(unix))]
async fn shutdown_signal() -> std::io::Result<()> {
    tokio::signal::ctrl_c().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use rcgen::generate_simple_self_signed;

    fn required_args() -> Vec<String> {
        vec![
            "dodb-server".to_owned(),
            "--data-dir".to_owned(),
            "/tmp/dodb-data".to_owned(),
            "--tls-cert".to_owned(),
            "/tmp/server.crt".to_owned(),
            "--tls-key".to_owned(),
            "/tmp/server.key".to_owned(),
        ]
    }

    #[test]
    fn cli_defaults_are_stable() {
        let args = Args::try_parse_from(required_args()).unwrap();

        assert_eq!(args.listen, "127.0.0.1:18445".parse().unwrap());
        assert_eq!(args.max_open_shards, 1024);
        assert_eq!(args.max_connections, 8);
        assert_eq!(args.max_concurrent_streams, 64);
        assert_eq!(args.max_concurrent_requests, 64);
    }

    #[test]
    fn cli_overrides_are_applied() {
        let mut cli_args = required_args();
        cli_args.extend([
            "--listen".to_owned(),
            "0.0.0.0:18446".to_owned(),
            "--max-open-shards".to_owned(),
            "12".to_owned(),
            "--max-connections".to_owned(),
            "3".to_owned(),
            "--max-concurrent-streams".to_owned(),
            "5".to_owned(),
            "--max-concurrent-requests".to_owned(),
            "7".to_owned(),
        ]);
        let args = Args::try_parse_from(cli_args).unwrap();

        assert_eq!(args.listen, "0.0.0.0:18446".parse().unwrap());
        assert_eq!(args.max_open_shards, 12);
        assert_eq!(args.max_connections, 3);
        assert_eq!(args.max_concurrent_streams, 5);
        assert_eq!(args.max_concurrent_requests, 7);
    }

    #[test]
    fn required_arguments_must_be_present() {
        assert!(Args::try_parse_from(["dodb-server"]).is_err());
    }

    #[test]
    fn zero_limits_are_rejected() {
        let limit_arguments = [
            "--max-open-shards",
            "--max-connections",
            "--max-concurrent-streams",
            "--max-concurrent-requests",
        ];

        for limit_argument in limit_arguments {
            let mut cli_args = required_args();
            cli_args.extend([limit_argument.to_owned(), "0".to_owned()]);
            let args = Args::try_parse_from(cli_args).unwrap();

            assert!(args.validate().is_err(), "{limit_argument} accepted zero");
        }
    }

    #[tokio::test]
    async fn server_binds_from_pem_files_and_shuts_down_cleanly() {
        let directory = tempfile::tempdir().unwrap();
        let certificate = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let certificate_path = directory.path().join("server.crt");
        let private_key_path = directory.path().join("server.key");
        std::fs::write(&certificate_path, certificate.cert.pem()).unwrap();
        std::fs::write(&private_key_path, certificate.signing_key.serialize_pem()).unwrap();
        let data_dir = directory.path().join("data");
        let cli_args = vec![
            "dodb-server".to_owned(),
            "--listen".to_owned(),
            "127.0.0.1:0".to_owned(),
            "--data-dir".to_owned(),
            data_dir.to_string_lossy().into_owned(),
            "--tls-cert".to_owned(),
            certificate_path.to_string_lossy().into_owned(),
            "--tls-key".to_owned(),
            private_key_path.to_string_lossy().into_owned(),
        ];
        let args = Args::try_parse_from(cli_args).unwrap();
        let server = Arc::new(build_server(&args).unwrap());
        assert_ne!(server.local_addr().unwrap().port(), 0);

        let server_task = tokio::spawn({
            let server = Arc::clone(&server);
            async move { server.run().await }
        });
        server.shutdown().await;

        assert!(server_task.await.unwrap().is_ok());
        assert!(data_dir.is_dir());
    }
}
