#![cfg(feature = "webhook")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;
use sologger_log_transport::webhook_config::{WebhookConfig, WebhookFormat};
use sologger_log_transport::webhook_lib::WebhookTransport;

/// One-shot HTTP server: accepts a single request on a random port, sends back the
/// given status, and returns the raw request (headers + body) through the channel.
fn one_shot_server(status_line: &'static str) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().unwrap();
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        // Read headers
        while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
        }
        // Read the body per Content-Length
        let text = String::from_utf8_lossy(&raw).to_string();
        let content_length: usize = text
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())?
            })
            .unwrap_or(0);
        let header_end = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
        while raw.len() < header_end + content_length {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
        }

        stream
            .write_all(format!("{}\r\ncontent-length: 0\r\n\r\n", status_line).as_bytes())
            .unwrap();
        sender.send(String::from_utf8_lossy(&raw).to_string()).unwrap();
    });

    (url, receiver)
}

fn failing_context() -> Vec<LogContext> {
    let logs: Vec<String> = vec![
        "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR invoke [1]",
        "Program log: Instruction: OpenPosition",
        "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR failed: custom program error: 0x1",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    LogContext::parse_logs(
        &logs,
        "".to_string(),
        &ProgramsSelector::new_all_programs(),
        42,
        "TESTSIG".to_string(),
    )
}

#[tokio::test]
async fn posts_discord_payload_to_endpoint() {
    let (url, received) = one_shot_server("HTTP/1.1 204 No Content");
    let transport = WebhookTransport::new(WebhookConfig {
        url,
        format: WebhookFormat::Discord,
        errors_only: true,
        ..Default::default()
    })
    .unwrap();

    let sent = transport.send_all(&failing_context()).await;
    assert_eq!(sent, 1);

    let request = received.recv().unwrap();
    assert!(request.starts_with("POST /hook"));
    assert!(request.contains("content-type: application/json"));
    assert!(request.contains("\"content\""));
    assert!(request.contains("OpenPosition"));
}

#[tokio::test]
async fn non_success_status_is_an_error() {
    let (url, _received) = one_shot_server("HTTP/1.1 500 Internal Server Error");
    let transport = WebhookTransport::new(WebhookConfig {
        url,
        ..Default::default()
    })
    .unwrap();

    let result = transport.send_payload("{}".to_string()).await;
    assert!(result.is_err());
    assert!(format!("{:#}", result.unwrap_err()).contains("500"));
}

#[tokio::test]
async fn send_all_swallows_delivery_failures() {
    // Nothing is listening on this port
    let transport = WebhookTransport::new(WebhookConfig {
        url: "http://127.0.0.1:1/hook".to_string(),
        timeout_ms: 500,
        ..Default::default()
    })
    .unwrap();

    let sent = transport.send_all(&failing_context()).await;
    assert_eq!(sent, 0);
}
