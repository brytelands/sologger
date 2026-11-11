#![cfg(feature = "otel")]
//! The providers sologger installs buffer telemetry: batched logs and spans, and metrics
//! between periodic exports. `OtelProviders::shutdown` must deliver what is buffered, or
//! every stop or restart silently drops it.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opentelemetry::logs::{LogRecord as _, Logger as _, LoggerProvider as _};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{Tracer as _, TracerProvider as _};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{self, BatchLogProcessor, LogBatch, LogExporter, SdkLoggerProvider};
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{
    self, BatchSpanProcessor, SdkTracerProvider, SpanData, SpanExporter,
};
use sologger_log_transport::opentelemetry_lib::OtelProviders;

/// Counts exported items. The SDK's in-memory log/span exporters clear their records
/// on shutdown, which would hide exactly what this test needs to see.
#[derive(Clone, Debug, Default)]
struct Counter(Arc<Mutex<usize>>);

impl Counter {
    fn add(&self, n: usize) {
        *self.0.lock().unwrap() += n;
    }
    fn get(&self) -> usize {
        *self.0.lock().unwrap()
    }
}

#[derive(Clone, Debug, Default)]
struct CountingLogExporter(Counter);

impl LogExporter for CountingLogExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        self.0.add(batch.iter().count());
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct CountingSpanExporter(Counter);

impl SpanExporter for CountingSpanExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        self.0.add(batch.len());
        Ok(())
    }
}

/// Long enough that nothing exports on its own during the test
const NEVER: Duration = Duration::from_secs(3600);

#[test]
fn shutdown_delivers_buffered_telemetry() {
    let log_exporter = CountingLogExporter::default();
    let logs = SdkLoggerProvider::builder()
        .with_log_processor(
            BatchLogProcessor::builder(log_exporter.clone())
                .with_batch_config(
                    logs::BatchConfigBuilder::default()
                        .with_scheduled_delay(NEVER)
                        .build(),
                )
                .build(),
        )
        .build();

    let span_exporter = CountingSpanExporter::default();
    let traces = SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(span_exporter.clone())
                .with_batch_config(
                    trace::BatchConfigBuilder::default()
                        .with_scheduled_delay(NEVER)
                        .build(),
                )
                .build(),
        )
        .build();

    let metric_exporter = InMemoryMetricExporter::default();
    let metrics = SdkMeterProvider::builder()
        .with_reader(
            PeriodicReader::builder(metric_exporter.clone())
                .with_interval(NEVER)
                .build(),
        )
        .build();

    let logger = logs.logger("test");
    let mut record = logger.create_log_record();
    record.set_body("buffered".into());
    logger.emit(record);
    traces.tracer("test").in_span("buffered", |_| {});
    metrics
        .meter("test")
        .u64_counter("buffered")
        .build()
        .add(1, &[]);

    assert_eq!(log_exporter.0.get(), 0, "log exported before shutdown");
    assert_eq!(span_exporter.0.get(), 0, "span exported before shutdown");

    let providers = OtelProviders {
        logs: Some(logs),
        traces: Some(traces),
        metrics: Some(metrics),
    };
    let errors = providers.shutdown();

    assert!(errors.is_empty(), "shutdown errors: {:?}", errors);
    assert_eq!(log_exporter.0.get(), 1, "buffered log was not delivered");
    assert_eq!(span_exporter.0.get(), 1, "buffered span was not delivered");
    let exported = metric_exporter.get_finished_metrics().unwrap();
    assert!(
        !exported.is_empty(),
        "buffered metric was not delivered on shutdown"
    );
}

#[test]
fn shutdown_without_providers_is_a_no_op() {
    assert!(OtelProviders::default().shutdown().is_empty());
}
