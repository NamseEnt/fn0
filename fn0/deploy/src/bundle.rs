use anyhow::{Result, anyhow};
use std::path::Path;

pub fn read_env_yaml(project_dir: &Path) -> Result<Option<Vec<u8>>> {
    let p = project_dir.join("env.yaml");
    match std::fs::read(&p) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow!("Failed to read {}: {}", p.display(), e)),
    }
}

pub fn create_raw_bundle_wasm(
    wasm_path: &Path,
    env_yaml: Option<&[u8]>,
    output_path: &Path,
) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);
    append_bytes(&mut builder, "manifest.json", br#"{"kind":"wasm"}"#)?;
    let wasm_bytes = std::fs::read(wasm_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", wasm_path.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;
    if let Some(env) = env_yaml {
        append_bytes(&mut builder, "env.yaml", env)?;
    }
    builder.finish()?;
    Ok(())
}

pub fn create_raw_bundle_forte(
    dist_dir: &Path,
    env_yaml: Option<&[u8]>,
    output_path: &Path,
) -> Result<()> {
    let file = std::fs::File::create(output_path)
        .map_err(|e| anyhow!("Failed to create {}: {}", output_path.display(), e))?;
    let mut builder = tar::Builder::new(file);
    append_bytes(&mut builder, "manifest.json", br#"{"kind":"wasmjs"}"#)?;

    let backend_wasm = dist_dir.join("backend.wasm");
    let wasm_bytes = std::fs::read(&backend_wasm)
        .map_err(|e| anyhow!("Failed to read {}: {}", backend_wasm.display(), e))?;
    append_bytes(&mut builder, "backend.wasm", &wasm_bytes)?;

    let server_js = dist_dir.join("server.js");
    let server_bytes = std::fs::read(&server_js)
        .map_err(|e| anyhow!("Failed to read {}: {}", server_js.display(), e))?;
    append_bytes(&mut builder, "entry.js", &server_bytes)?;

    if let Some(env) = env_yaml {
        append_bytes(&mut builder, "env.yaml", env)?;
    }

    builder.finish()?;
    Ok(())
}

fn append_bytes<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    path: &str,
    data: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, data)
        .map_err(|e| anyhow!("tar append failed for {}: {}", path, e))?;
    Ok(())
}
