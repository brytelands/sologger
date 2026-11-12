#![cfg(feature = "enable_webhook")]
//! A `webhookConfigLocation` that is set but unusable disables the webhook transport with a
//! clear notice, like the other transports; it used to panic at startup.

use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

fn closed_local_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Starts sologger with `webhook_location`, waits briefly, and returns whether it was
/// still running plus everything it wrote to stderr.
fn run_with_webhook_config(dir: &Path, webhook_location: &Path) -> (bool, String) {
    let config = dir.join("sologger-config.json");
    std::fs::write(
        &config,
        serde_json::json!({
            "webhookConfigLocation": webhook_location,
            "rpcUrl": format!("ws://127.0.0.1:{}", closed_local_port()),
            "programsSelector": { "programs": ["*"] }
        })
        .to_string(),
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_sologger"))
        .arg(&config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
    let running = child.try_wait().unwrap().is_none();
    if running {
        child.kill().unwrap();
    }
    let stderr = String::from_utf8_lossy(&child.wait_with_output().unwrap().stderr).into_owned();
    (running, stderr)
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sologger-{}-{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_missing_webhook_config_disables_the_transport_instead_of_panicking() {
    let dir = temp_dir("webhook-missing");
    let missing = dir.join("no-such-webhook-config.json");

    let (running, stderr) = run_with_webhook_config(&dir, &missing);
    std::fs::remove_dir_all(&dir).ok();

    assert!(running, "sologger exited:\n{}", stderr);
    assert!(
        stderr.contains("webhook transport disabled")
            && stderr.contains("no-such-webhook-config.json"),
        "no clear notice:\n{}",
        stderr
    );
}

#[test]
fn an_invalid_webhook_config_disables_the_transport_instead_of_panicking() {
    let dir = temp_dir("webhook-invalid");
    let webhook = dir.join("webhook-config.json");
    std::fs::write(&webhook, "{ not json").unwrap();

    let (running, stderr) = run_with_webhook_config(&dir, &webhook);
    std::fs::remove_dir_all(&dir).ok();

    assert!(running, "sologger exited:\n{}", stderr);
    assert!(
        stderr.contains("webhook transport disabled") && stderr.contains("webhook-config.json"),
        "no clear notice:\n{}",
        stderr
    );
}
