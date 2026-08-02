//! Runtime switchboard for OTel traces and metrics, populated during logger init from
//! the `enableTraces` / `enableMetrics` flags in the OpenTelemetry config. When neither
//! is enabled, `export` is a cheap no-op.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use sologger_log_context::sologger_log_context::LogContext;
use sologger_log_transport::solana_telemetry::{record_transaction_trace, SologgerMetrics};

static TRACES_ENABLED: AtomicBool = AtomicBool::new(false);
static METRICS: OnceLock<SologgerMetrics> = OnceLock::new();

/// Start exporting transaction traces. Call after `init_tracer` has installed the
/// global tracer provider.
pub fn enable_traces() {
    TRACES_ENABLED.store(true, Ordering::Relaxed);
}

/// Build the metric instruments. Call after `init_metrics` has installed the global
/// meter provider.
pub fn enable_metrics() {
    METRICS.get_or_init(SologgerMetrics::new);
}

/// Exports traces and metrics for one parsed batch, according to what was enabled.
pub fn export(log_contexts: &[LogContext]) {
    if log_contexts.is_empty() {
        return;
    }
    if TRACES_ENABLED.load(Ordering::Relaxed) {
        record_transaction_trace(log_contexts);
    }
    if let Some(metrics) = METRICS.get() {
        metrics.record(log_contexts);
    }
}

/// Records a WebSocket reconnect, when metrics are enabled.
#[allow(dead_code)]
pub fn record_reconnect() {
    if let Some(metrics) = METRICS.get() {
        metrics.record_reconnect();
    }
}
