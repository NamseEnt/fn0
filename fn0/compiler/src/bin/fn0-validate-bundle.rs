//! Answers whether a stored bundle would survive a wasmtime bump, using the
//! same wasmparser the platform validates uploads with.
//!
//! Accepts either a raw `.wasm` or an `original/` bundle tar, whose
//! `backend.wasm` is the component the runtime loads.

use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: fn0-validate-bundle <bundle.tar|component.wasm>");
        return ExitCode::from(2);
    };

    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::from(2);
        }
    };

    let component = match backend_wasm(&bytes) {
        Ok(component) => component,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::from(2);
        }
    };

    match wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&component)
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{path}: {e}");
            ExitCode::from(1)
        }
    }
}

fn backend_wasm(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.starts_with(b"\0asm") {
        return Ok(bytes.to_vec());
    }

    let mut archive = tar::Archive::new(bytes);
    let entries = archive
        .entries()
        .map_err(|e| format!("not a wasm file and not a readable tar: {e}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("tar entry: {e}"))?;
        let is_backend = entry
            .path()
            .map(|path| path.to_string_lossy() == "backend.wasm")
            .unwrap_or(false);
        if is_backend {
            let mut component = Vec::new();
            entry
                .read_to_end(&mut component)
                .map_err(|e| format!("read backend.wasm: {e}"))?;
            return Ok(component);
        }
    }
    Err("bundle has no backend.wasm".to_string())
}
