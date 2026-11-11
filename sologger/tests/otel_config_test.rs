#![cfg(feature = "enable_otel")]
//! An OpenTelemetry config that exists but is broken disables that transport with a clear
//! message; it used to panic at startup.

use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::Duration;

fn closed_local_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn a_broken_otel_config_disables_the_transport_instead_of_panicking() {
    let dir = std::env::temp_dir().join(format!("sologger-otel-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let otel = dir.join("opentelemetry-config.json");
    std::fs::write(&otel, r#"{ "logLevel": "INFO" }"#).unwrap(); // no `endpoint`
    let config = dir.join("sologger-config.json");
    std::fs::write(
        &config,
        serde_json::json!({
            "opentelemetryConfigLocation": otel,
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
    let exited = child.try_wait().unwrap();
    if exited.is_none() {
        child.kill().unwrap();
    }
    let stderr = String::from_utf8_lossy(&child.wait_with_output().unwrap().stderr).into_owned();
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        exited.is_none(),
        "sologger exited ({:?}):\n{}",
        exited,
        stderr
    );
    assert!(
        stderr.contains("otel transport disabled") && stderr.contains("endpoint"),
        "no clear notice:\n{}",
        stderr
    );
    assert!(stderr.contains("pretty console output"), "{}", stderr);
}
