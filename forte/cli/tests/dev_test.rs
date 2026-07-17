use assert_cmd::cargo;
use std::io::{BufRead, BufReader};
use std::process::{Child, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn get_forte_bin_path() -> std::path::PathBuf {
    cargo::cargo_bin!("forte").to_path_buf()
}

struct DevServer {
    child: Child,
    port: u16,
    _stdout_thread: std::thread::JoinHandle<()>,
}

impl DevServer {
    fn start(project_dir: &std::path::Path) -> Self {
        let forte_bin = get_forte_bin_path();

        let mut child = std::process::Command::new(&forte_bin)
            .args(["dev"])
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start forte dev");

        let stdout = child.stdout.take().expect("Failed to get stdout");
        let (tx, rx) = mpsc::channel::<u16>();

        let stdout_thread = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut sent = false;

            for line in reader.lines() {
                let Ok(line) = line else { break };
                eprintln!("[dev server] {}", line);

                if !sent
                    && line.contains("Listening on")
                    && let Some(port_str) = line.split(':').next_back()
                    && let Ok(forte_port) = port_str.trim().parse()
                {
                    let _ = tx.send(forte_port);
                    sent = true;
                }
            }
        });

        let port = rx
            .recv_timeout(Duration::from_secs(120))
            .expect("Timeout waiting for server to start");

        Self {
            child,
            port,
            _stdout_thread: stdout_thread,
        }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for DevServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn init_project(temp_dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    cargo::cargo_bin_cmd!("forte")
        .args(["init", name])
        .current_dir(temp_dir)
        .assert()
        .success();

    temp_dir.join(name)
}

fn install_npm_deps(project_dir: &std::path::Path) {
    std::process::Command::new("npm")
        .arg("install")
        .current_dir(project_dir.join("fe"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run npm install");
}

#[test]
fn test_dev_server_starts_and_responds() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = init_project(temp.path(), "test-app");

    install_npm_deps(&project_dir);

    let index_page_path = project_dir.join("fe/src/pages/index/page.tsx");
    let index_page = std::fs::read_to_string(&index_page_path).unwrap();
    std::fs::write(
        &index_page_path,
        format!(
            "{index_page}\nexport function head(_props: Props) {{\n    return [\n        {{ title: \"Index Head Title\" }},\n        {{ name: \"description\", content: \"index description\" }},\n    ];\n}}\n"
        ),
    )
    .unwrap();

    let server = DevServer::start(&project_dir);

    std::thread::sleep(Duration::from_secs(1));

    let response = reqwest::blocking::get(server.url());

    match response {
        Ok(resp) => {
            assert!(
                resp.status().is_success(),
                "Expected success status, got {}",
                resp.status()
            );
            let body = resp.text().unwrap();
            assert!(body.contains("html"), "Expected HTML response");
            assert!(
                body.contains("<title>Index Head Title</title>"),
                "Expected page head title, got: {body}"
            );
            assert!(
                !body.contains("<title>Forte App</title>"),
                "App default title must be overridden by the page head, got: {body}"
            );
            assert!(
                body.contains("name=\"viewport\""),
                "App head defaults without page overrides must be kept, got: {body}"
            );
            assert!(
                body.contains("name=\"description\" content=\"index description\""),
                "Expected page head meta, got: {body}"
            );
        }
        Err(e) => {
            panic!("Failed to connect to dev server: {}", e);
        }
    }
}

#[test]
fn test_dev_auto_selects_port_if_busy() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = init_project(temp.path(), "test-app-2");

    install_npm_deps(&project_dir);

    let _listener = std::net::TcpListener::bind("0.0.0.0:3000").unwrap();

    let server = DevServer::start(&project_dir);

    std::thread::sleep(Duration::from_secs(1));

    assert_ne!(server.port, 3000, "Should have selected a different port");

    let response = reqwest::blocking::get(server.url());
    assert!(response.is_ok(), "Server should respond on alternate port");
}

fn vite_ssr_server_running(project_dir: &std::path::Path) -> bool {
    let pattern = project_dir.join("fe/.forte/dev/vite-ssr-server-");
    std::process::Command::new("pgrep")
        .args(["-f", &pattern.to_string_lossy()])
        .output()
        .expect("Failed to run pgrep")
        .status
        .success()
}

#[test]
fn test_vite_ssr_exits_when_forte_dies() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = init_project(temp.path(), "test-app-vite-ssr-exit");

    install_npm_deps(&project_dir);

    let mut server = DevServer::start(&project_dir);

    std::thread::sleep(Duration::from_secs(1));

    assert!(
        vite_ssr_server_running(&project_dir),
        "Vite SSR adapter process should be running"
    );

    server.child.kill().expect("Failed to kill forte");
    server.child.wait().expect("Failed to wait for forte");

    let mut vite_ssr_closed = false;
    for _ in 0..30 {
        std::thread::sleep(Duration::from_millis(100));
        if !vite_ssr_server_running(&project_dir) {
            vite_ssr_closed = true;
            break;
        }
    }

    assert!(
        vite_ssr_closed,
        "Vite SSR adapter should have exited after forte died"
    );
}
