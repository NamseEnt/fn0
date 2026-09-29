use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use rcgen::generate_simple_self_signed;
use tempfile::tempdir;

#[test]
fn binary_starts_without_an_otlp_receiver_in_a_fresh_process() {
    let directory = tempdir().expect("create temporary directory");
    let certificate = generate_simple_self_signed(vec!["localhost".to_owned()])
        .expect("generate temporary TLS certificate");
    let certificate_path = directory.path().join("server.crt");
    let private_key_path = directory.path().join("server.key");
    let data_directory = directory.path().join("data");
    std::fs::write(&certificate_path, certificate.cert.pem()).expect("write TLS certificate");
    std::fs::write(&private_key_path, certificate.signing_key.serialize_pem())
        .expect("write TLS private key");

    let mut child = Command::new(env!("CARGO_BIN_EXE_dodb-server"))
        .arg("--listen")
        .arg("127.0.0.1:0")
        .arg("--data-dir")
        .arg(&data_directory)
        .arg("--tls-cert")
        .arg(&certificate_path)
        .arg("--tls-key")
        .arg(&private_key_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start dodb-server subprocess");

    let (output_sender, output_receiver) = mpsc::channel();
    forward_lines(
        child.stdout.take().expect("capture stdout"),
        output_sender.clone(),
    );
    forward_lines(
        child.stderr.take().expect("capture stderr"),
        output_sender.clone(),
    );
    drop(output_sender);

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();
    let mut serving = false;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("check child process") {
            panic!("dodb-server exited before serving with {status}; output: {output:?}");
        }
        match output_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                serving |= line.contains("dodb-server listening on 127.0.0.1:");
                output.push(line);
                if serving {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if !serving {
        let _ = child.kill();
        let _ = child.wait();
        panic!("dodb-server did not reach serving state; output: {output:?}");
    }
    assert!(
        output.iter().all(|line| !line.contains("No provider set")),
        "dodb-server panicked because no rustls provider was installed: {output:?}"
    );

    let terminate_status = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status()
        .expect("send SIGTERM to dodb-server");
    assert!(terminate_status.success(), "SIGTERM command failed");
    let shutdown_deadline = Instant::now() + Duration::from_secs(10);
    let shutdown_status = loop {
        if let Some(status) = child.try_wait().expect("check graceful server shutdown") {
            break status;
        }
        if Instant::now() >= shutdown_deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("dodb-server did not terminate after SIGTERM; output: {output:?}");
        }
        thread::sleep(Duration::from_millis(50));
    };
    while let Ok(line) = output_receiver.recv_timeout(Duration::from_millis(100)) {
        output.push(line);
    }
    assert!(
        output.iter().all(|line| !line.contains("No provider set")),
        "dodb-server panicked because no rustls provider was installed: {output:?}"
    );
    assert!(
        output.iter().all(|line| !line.contains("panicked at")),
        "dodb-server panicked during startup or shutdown: {output:?}"
    );
    assert!(
        output
            .iter()
            .any(|line| line.contains("dodb telemetry baseline flush failed")),
        "missing OTLP receiver did not exercise the best-effort baseline flush path: {output:?}"
    );
    assert!(
        shutdown_status.code().is_some(),
        "server shutdown was not observed"
    );
}

fn forward_lines<R: Read + Send + 'static>(output: R, output_sender: Sender<String>) {
    thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            let Ok(line) = line else {
                break;
            };
            if output_sender.send(line).is_err() {
                break;
            }
        }
    });
}
