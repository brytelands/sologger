//! Binary-side holder for the webhook transport. Initialized once from
//! `webhookConfigLocation` in sologger-config.json; the subscriber loop hands each
//! parsed batch to [`dispatch`], which sends matching records on a background task so
//! webhook latency never stalls log ingestion.

use std::sync::{Arc, OnceLock};

use sologger_log_context::sologger_log_context::LogContext;
use sologger_log_transport::webhook_lib::WebhookTransport;

use crate::sologger_config::SologgerConfig;

static TRANSPORT: OnceLock<Arc<WebhookTransport>> = OnceLock::new();

/// Builds the webhook transport from the configured location. An empty location leaves
/// the webhook disabled with a notice; a location that exists but fails to load is a
/// hard config error, matching how the other transports treat broken configs.
pub fn init(sologger_config: &SologgerConfig) {
    let location = &sologger_config.webhook_config_location;
    if location.is_empty() {
        eprintln!("sologger: webhookConfigLocation not set — webhook transport disabled");
        return;
    }
    let transport = WebhookTransport::from_config_path(location)
        .expect("Failed to load webhook config");
    let _ = TRANSPORT.set(Arc::new(transport));
}

/// Sends the matching records of a parsed batch, if a webhook is configured. Payloads
/// are built synchronously (cheap) and delivered on a spawned task.
pub fn dispatch(log_contexts: &[LogContext]) {
    let Some(transport) = TRANSPORT.get() else {
        return;
    };
    let payloads = transport.matching_payloads(log_contexts);
    if payloads.is_empty() {
        return;
    }
    let transport = Arc::clone(transport);
    tokio::spawn(async move {
        for payload in payloads {
            if let Err(err) = transport.send_payload(payload).await {
                log::warn!("webhook delivery failed: {}", err);
            }
        }
    });
}
