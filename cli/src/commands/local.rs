use color_eyre::Result;
use std::path::PathBuf;

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

    fn0::run(config, |port| {
        println!("Server on http://localhost:{}", port);
    })
    .await?;

    Ok(())
}
