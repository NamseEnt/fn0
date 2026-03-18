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

fn derive_sysroot(rustc_path: &str) -> String {
    let output = Command::new(rustc_path)
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("Failed to get sysroot from rustc");
    String::from_utf8(output.stdout)
        .expect("Invalid sysroot output")
        .trim()
        .to_string()
}

fn main() {
    if env::var("MY_ANALYZER_WRAPPER_MODE").is_ok() {
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

        let mut compiler_args: Vec<String> = args[1..].to_vec();
        if !compiler_args.iter().any(|a| a == "--sysroot") {
            let sysroot = derive_sysroot(&args[1]);
            compiler_args.push("--sysroot".to_string());
            compiler_args.push(sysroot);
        }

        run_compiler(
            &compiler_args,
            &mut Analyzer {
                ts_output_dir: "../fe/src/pages".to_string(),
                hooks_output_dir: "../fe/src/hooks/.generated".to_string(),
                actions_output_dir: "../fe/src/actions/.generated".to_string(),
            },
        );
        return;
    }
    let project_dir = env::args()
        .nth(1)
        .expect("Usage: forte-rs-to-ts <project-dir>");
    let target_dir = format!("{}/rs", project_dir);

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
