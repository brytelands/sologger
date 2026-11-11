#![cfg(all(feature = "enable_logstash", feature = "enable_otel"))]
//! A binary with both the logstash and OpenTelemetry transports, given both configs, must
//! run with both active. Both used to install the process-wide logger, so the second
//! one panicked at startup.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn closed_local_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn logstash_and_otel_run_together() {
    let dir = std::env::temp_dir().join(format!("sologger-combined-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // Stand-in for LogStash: collects the JSON lines the logstash appender sends
    let logstash = TcpListener::bind("127.0.0.1:0").unwrap();
    let logstash_port = logstash.local_addr().unwrap().port();
    let (lines_tx, lines_rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok((stream, _)) = logstash.accept() {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let _ = lines_tx.send(line);
            }
        }
    });

    let log4rs = dir.join("log4rs-config.yml");
    std::fs::write(
        &log4rs,
        format!(
            "appenders:\n  logstash:\n    kind: logstash\n    hostname: 127.0.0.1\n    port: {logstash_port}\n    buffer_size: 1\n    buffer_lifetime: 100ms\n\
             root:\n  level: INFO\n  appenders:\n    - logstash\n"
        ),
    )
    .unwrap();

    // Nothing listens there; the OTel exporters just retry in the background
    let otel = dir.join("opentelemetry-config.json");
    std::fs::write(
        &otel,
        serde_json::json!({
            "logConfig": { "service.name": "combined-test" },
            "endpoint": format!("http://127.0.0.1:{}", closed_local_port()),
            "metricsEndpoint": "",
            "tracesEndpoint": "",
            "logLevel": "INFO",
            "enableTraces": true,
            "enableMetrics": true
        })
        .to_string(),
    )
    .unwrap();

    // An unreachable RPC makes the supervisor log reconnect warnings without network access
    let config = dir.join("sologger-config.json");
    std::fs::write(
        &config,
        serde_json::json!({
            "log4rsConfigLocation": log4rs,
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

    let received = lines_rx.recv_timeout(Duration::from_secs(10));
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
    let line = received.unwrap_or_else(|_| panic!("logstash received nothing:\n{}", stderr));
    assert!(
        line.contains("subscription error"),
        "unexpected record: {line}"
    );
}
