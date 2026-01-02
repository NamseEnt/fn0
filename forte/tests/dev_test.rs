use assert_cmd::Command;
use std::io::{BufRead, BufReader};
use std::process::{Child, Stdio};
use std::time::Duration;

fn get_forte_bin_path() -> std::path::PathBuf {
    Command::cargo_bin("forte").unwrap().get_program().to_path_buf()
}

struct DevServer {
    child: Child,
    port: u16,
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
        let reader = BufReader::new(stdout);

        let mut port = 3000u16;
        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            eprintln!("[dev server] {}", line);
            if line.contains("listening on") {
                if let Some(port_str) = line.split(':').last() {
                    port = port_str.trim().parse().unwrap_or(3000);
                }
                break;
            }
        }

        Self { child, port }
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
    Command::cargo_bin("forte")
        .unwrap()
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

    let server = DevServer::start(&project_dir);

    std::thread::sleep(Duration::from_secs(1));

    let response = reqwest::blocking::get(&server.url());

    match response {
        Ok(resp) => {
            assert!(
                resp.status().is_success(),
                "Expected success status, got {}",
                resp.status()
            );
            let body = resp.text().unwrap();
            assert!(body.contains("html"), "Expected HTML response");
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

    let _listener = std::net::TcpListener::bind("127.0.0.1:3000").unwrap();

    let server = DevServer::start(&project_dir);

    std::thread::sleep(Duration::from_secs(1));

    assert_ne!(server.port, 3000, "Should have selected a different port");

    let response = reqwest::blocking::get(&server.url());
    assert!(response.is_ok(), "Server should respond on alternate port");
}
