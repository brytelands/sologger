use std::fs;
use std::str::FromStr;

use anyhow::Result as AnyResult;
use log::{Level, trace};
use opentelemetry::{Key, KeyValue, metrics, trace::Tracer, global};
use opentelemetry::logs::LogError;
use opentelemetry::metrics::{MeterProvider, MetricsError};
use opentelemetry::metrics::noop::NoopMeterProvider;
use opentelemetry::trace::TraceError;
use opentelemetry_api::global::logger_provider;
use opentelemetry_api::trace::FutureExt;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, runtime, trace as sdktrace};
use opentelemetry_sdk::logs::{BatchConfig, LoggerProvider};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{Config, TracerProvider};
use opentelemetry_stdout::SpanExporter;
use serde_json;

use crate::opentelemetry_config::OpentelemetryConfig;

/// Initialize the logger with the provided logstash config location
pub fn init_logs_opentelemetry_with_config_path(
    path: &String,
) -> AnyResult<LoggerProvider, LogError> {
    let config = get_otel_config(path);

    init_logs_opentelemetry(&config)
}

/// Initialize the logger with the provided logstash config
#[cfg(feature = "otel")]
pub fn init_logs_opentelemetry(
    config: &OpentelemetryConfig,
) -> AnyResult<LoggerProvider, LogError> {
    let log_config: Vec<KeyValue> = config
        .log_config
        .iter()
        .map(|(k, v)| KeyValue::new(Key::new(k.clone()), v.to_string()))
        .collect();
    trace!("OLTP log_config: {:?}", log_config);

    let logger = opentelemetry_otlp::new_pipeline()
        .logging().with_resource(Resource::new(log_config))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.endpoint),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to initialize opentelemetry logging");

    // Create a new OpenTelemetryLogBridge using the above LoggerProvider.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();
    log::set_max_level(
        Level::from_str(&config.log_level)
            .unwrap_or(Level::Error)
            .to_level_filter(),
    );
    Ok(logger)
}

// TODO add configuration
#[cfg(feature = "otel")]
pub fn init_tracer(config: &OpentelemetryConfig) -> Result<TracerProvider, TraceError> {
    let provider = TracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    
    Ok(provider)
}

// TODO add configuration
#[cfg(feature = "otel")]
pub fn init_metrics(config: &OpentelemetryConfig) -> Result<SdkMeterProvider, MetricsError> {
    let exporter = opentelemetry_stdout::MetricsExporterBuilder::default().build();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new([KeyValue::new(
            "service.name",
            "metrics-basic-example",
        )]))
        .build();
    global::set_meter_provider(provider.clone());
    Ok(provider)
}

/// Returns the OpentelemetryConfig struct converted from the otel config json
pub fn get_otel_config(path: &String) -> OpentelemetryConfig {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let config: OpentelemetryConfig = serde_json::from_str(&contents).unwrap();
    config
}
