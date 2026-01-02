#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod analyzer;
mod name_resolution;
mod rust_to_ts;
mod ts_codegen;

use analyzer::Analyzer;
use rustc_driver::run_compiler;
use std::env;
use std::process::Command;

fn main() {
    if env::var("MY_ANALYZER_WRAPPER_MODE").is_ok() {
        // args[0] = forte-rs-to-ts
        // args[1] = /path/to/.../rustc
        // args[2..] = --crate-name build_script_build ...
        let args: Vec<String> = env::args().collect();

        // IMPORTANT: DO NOT REMOVE THIS CHECK!
        // RUSTC_WORKSPACE_WRAPPER applies to build.rs compilation as well.
        // Without this early return, build scripts won't compile and cargo will fail with:
        // "could not execute process build-script-build (No such file or directory)"
        let is_build_script = args.iter().any(|arg| arg == "build_script_build");
        if is_build_script {
            let status = Command::new(&args[1])
                .args(&args[2..])
                .status()
                .expect("Failed to execute original rustc");
            std::process::exit(status.code().unwrap_or(1));
        }

        run_compiler(
            &args[1..],
            &mut Analyzer {
                ts_output_dir: "../fe/src/pages".to_string(),
            },
        );
        return;
    }
    let target_dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "../forte-manual/rs".to_string());

    let current_exe = env::current_exe().expect("Failed to find current exe");
    println!("Running cargo check on: {target_dir}");

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&target_dir)
        .env("RUSTC_WORKSPACE_WRAPPER", current_exe)
        .env("MY_ANALYZER_WRAPPER_MODE", "true")
        .status()
        .expect("Failed to run cargo");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
