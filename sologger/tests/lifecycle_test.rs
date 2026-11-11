//! Process lifecycle of the sologger binary: it must keep running without a terminal
//! attached (Docker without `-i`, systemd, nohup) and stop cleanly on SIGTERM.

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A loopback port with nothing listening, so the subscription supervisors keep
/// reconnecting instead of streaming — no network access needed.
fn closed_local_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn write_config(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("sologger-{}-{}.json", name, std::process::id()));
    let config = serde_json::json!({
        "rpcUrl": format!("ws://127.0.0.1:{}", closed_local_port()),
        "programsSelector": { "programs": ["*"] }
    });
    std::fs::write(&path, config.to_string()).unwrap();
    path
}

fn spawn_without_stdin(config: &std::path::Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_sologger"))
        .arg(config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start sologger")
}

fn wait_for_exit(child: &mut Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            return Some(status);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

fn stderr_of(child: Child) -> String {
    String::from_utf8_lossy(&child.wait_with_output().unwrap().stderr).into_owned()
}

#[test]
fn keeps_running_when_stdin_is_closed() {
    let config = write_config("no-stdin");
    let mut child = spawn_without_stdin(&config);

    let exited = wait_for_exit(&mut child, Duration::from_secs(3));
    if exited.is_none() {
        child.kill().unwrap();
    }
    let stderr = stderr_of(child);
    std::fs::remove_file(&config).ok();

    assert!(
        exited.is_none(),
        "sologger exited ({:?}) with stdin closed:\n{}",
        exited,
        stderr
    );
}

#[cfg(unix)]
#[test]
fn exits_cleanly_on_sigterm() {
    let config = write_config("sigterm");
    let mut child = spawn_without_stdin(&config);
    std::thread::sleep(Duration::from_millis(500));

    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());

    let exited = wait_for_exit(&mut child, Duration::from_secs(5));
    if exited.is_none() {
        child.kill().unwrap();
    }
    let stderr = stderr_of(child);
    std::fs::remove_file(&config).ok();

    let status = exited.unwrap_or_else(|| panic!("sologger ignored SIGTERM:\n{}", stderr));
    assert!(status.success(), "exit {:?}:\n{}", status, stderr);
}
