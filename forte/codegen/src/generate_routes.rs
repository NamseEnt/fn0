mod codegen;
mod discover;
mod model;

use codegen::{
    generate_actions_mod, generate_admin_mod, generate_code, generate_fe_paths,
    generate_queue_task_mod,
};
use discover::{
    discover_actions, discover_admin_tasks, discover_apis, discover_hooks, discover_pages,
    discover_public, discover_queue_tasks, discover_websockets,
};
use std::{env, fs, path::Path};

pub fn generate_routes() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set; call generate_routes from build.rs");
    let wit_dir = crate::extract_wit(Path::new(&out_dir));
    let wit_dir_str = wit_dir
        .to_str()
        .expect("wit dir path is not valid UTF-8")
        .to_string();

    let pages_dir = Path::new(&manifest_dir).join("src/pages");
    let apis_dir = Path::new(&manifest_dir).join("src/apis");
    let hooks_dir = Path::new(&manifest_dir).join("src/hooks");
    let actions_dir = Path::new(&manifest_dir).join("src/actions");
    let queue_task_dir = Path::new(&manifest_dir).join("src/queue_task");
    let admin_dir = Path::new(&manifest_dir).join("src/admin");
    let public_dir = Path::new(&manifest_dir).join("public");
    let ws_in_dir = Path::new(&manifest_dir).join("src/ws_in");
    let ws_out_dir = Path::new(&manifest_dir).join("src/ws_out");
    let output_path = Path::new(&manifest_dir).join("src/route_generated.rs");
    let fe_paths_output = Path::new(&manifest_dir).join("../fe/src/paths.generated.ts");

    println!("cargo:rerun-if-changed=src/pages");
    println!("cargo:rerun-if-changed=src/apis");
    println!("cargo:rerun-if-changed=src/hooks");
    println!("cargo:rerun-if-changed=src/actions");
    println!("cargo:rerun-if-changed=src/queue_task");
    println!("cargo:rerun-if-changed=src/admin");
    println!("cargo:rerun-if-changed=public");
    println!("cargo:rerun-if-changed=src/ws_in");
    println!("cargo:rerun-if-changed=src/ws_out");
    // Also rerun when dependency versions change (e.g. forte-sdk bump),
    // because once any rerun-if-changed is declared Cargo stops doing
    // default change detection across the rest of the crate.
    println!("cargo:rerun-if-changed=Cargo.lock");

    let mut pages = discover_pages(&pages_dir);
    pages.extend(discover_apis(&apis_dir));
    let hooks = discover_hooks(&hooks_dir);
    let actions = discover_actions(&actions_dir);
    let queue_tasks = discover_queue_tasks(&queue_task_dir);
    let admin_tasks = discover_admin_tasks(&admin_dir);
    let static_files = discover_public(&public_dir);
    let websockets = discover_websockets(&ws_in_dir, &ws_out_dir);
    let has_ws_out = websockets
        .iter()
        .any(|websocket| matches!(websocket.direction, model::WebSocketDirection::Outbound));
    let tokens = generate_code(
        &pages,
        &hooks,
        &actions,
        &queue_tasks,
        &admin_tasks,
        &static_files,
        &websockets,
        &wit_dir_str,
    );

    let syntax_tree = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated code");
    let pp_output = prettyplease::unparse(&syntax_tree);
    let formatted = run_rustfmt(&pp_output).unwrap_or(pp_output);

    let current_content = fs::read_to_string(&output_path).unwrap_or_default();
    if current_content != formatted {
        fs::write(&output_path, formatted).unwrap();
    }

    let fe_paths_content = generate_fe_paths(&pages);
    let current_fe_paths = fs::read_to_string(&fe_paths_output).unwrap_or_default();
    if current_fe_paths != fe_paths_content {
        fs::write(&fe_paths_output, fe_paths_content).unwrap();
    }

    if !queue_tasks.is_empty() {
        let queue_task_mod_path = Path::new(&manifest_dir).join("src/queue_task/mod.rs");
        let queue_task_mod_content = generate_queue_task_mod(&queue_tasks);
        let current_qt_mod = fs::read_to_string(&queue_task_mod_path).unwrap_or_default();
        if current_qt_mod != queue_task_mod_content {
            fs::write(&queue_task_mod_path, queue_task_mod_content).unwrap();
        }
    }

    if !admin_tasks.is_empty() {
        let admin_mod_path = Path::new(&manifest_dir).join("src/admin/mod.rs");
        let admin_mod_content = generate_admin_mod(&admin_tasks);
        let current_admin_mod = fs::read_to_string(&admin_mod_path).unwrap_or_default();
        if current_admin_mod != admin_mod_content {
            fs::write(&admin_mod_path, admin_mod_content).unwrap();
        }
    }

    if !actions.is_empty() {
        let actions_mod_path = Path::new(&manifest_dir).join("src/actions/mod.rs");
        let actions_mod_content = generate_actions_mod(&actions);
        let current_actions_mod = fs::read_to_string(&actions_mod_path).unwrap_or_default();
        if current_actions_mod != actions_mod_content {
            fs::write(&actions_mod_path, actions_mod_content).unwrap();
        }
    }

    let lib_rs_path = Path::new(&manifest_dir).join("src/lib.rs");
    update_lib_rs_managed_block(
        &lib_rs_path,
        !actions.is_empty(),
        !admin_tasks.is_empty(),
        !queue_tasks.is_empty(),
        has_ws_out,
    );
}

const LIB_RS_MARKER_START: &str = "// === FORTE-MANAGED START ===";
const LIB_RS_MARKER_END: &str = "// === FORTE-MANAGED END ===";

fn render_managed_block(
    has_actions: bool,
    has_admin: bool,
    has_queue_tasks: bool,
    has_ws_out: bool,
) -> String {
    let mut lines = Vec::new();
    lines.push(LIB_RS_MARKER_START.to_string());
    lines.push(
        "// Auto-managed by `forte build`. Do not edit between the START/END markers.".to_string(),
    );
    if has_actions {
        lines.push("pub mod actions;".to_string());
    }
    if has_admin {
        lines.push("pub mod admin;".to_string());
    }
    if has_queue_tasks {
        lines.push("pub mod queue_task;".to_string());
    }
    lines.push("mod route_generated;".to_string());
    if has_queue_tasks {
        lines.push("pub use route_generated::enqueue;".to_string());
    }
    if has_ws_out {
        lines.push("pub use route_generated::ws_out;".to_string());
    }
    lines.push(LIB_RS_MARKER_END.to_string());
    lines.join("\n")
}

fn update_lib_rs_managed_block(
    lib_rs_path: &Path,
    has_actions: bool,
    has_admin: bool,
    has_queue_tasks: bool,
    has_ws_out: bool,
) {
    let new_block = render_managed_block(has_actions, has_admin, has_queue_tasks, has_ws_out);
    let existing = fs::read_to_string(lib_rs_path).unwrap_or_default();

    let updated = match (
        existing.find(LIB_RS_MARKER_START),
        existing.find(LIB_RS_MARKER_END),
    ) {
        (Some(start), Some(end_marker)) if start < end_marker => {
            let block_end = end_marker + LIB_RS_MARKER_END.len();
            let mut out = String::with_capacity(existing.len() + new_block.len());
            out.push_str(&existing[..start]);
            out.push_str(&new_block);
            out.push_str(&existing[block_end..]);
            out
        }
        _ => {
            panic!(
                "src/lib.rs is missing the forte-managed marker block.\n\
                 Add the following lines to src/lib.rs (typically at the top), then re-run `forte build`:\n\n{new_block}\n",
            );
        }
    };

    if updated != existing {
        fs::write(lib_rs_path, updated).unwrap();
    }
}

fn run_rustfmt(input: &str) -> Option<String> {
    use std::io::Write;
    let mut child = std::process::Command::new("rustfmt")
        .arg("--edition=2024")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.as_mut()?.write_all(input.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
