use std::fs;
use std::str::FromStr;

use anyhow::Result as AnyResult;
use log::{Level, trace};
use opentelemetry_api::{Key, KeyValue, metrics, trace::Tracer};
use opentelemetry_api::global::logger_provider;
use opentelemetry_api::logs::LogError;
use opentelemetry_api::trace::TraceError;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, runtime, trace as sdktrace};
use opentelemetry_sdk::logs::Config;
use opentelemetry_sdk::metrics::MeterProvider;
use serde_json;

use crate::opentelemetry_config::OpentelemetryConfig;

/// Initialize the logger with the provided logstash config location
pub fn init_logs_opentelemetry_with_config_path(
    path: &String,
) -> AnyResult<opentelemetry_sdk::logs::Logger, LogError> {
    let config = get_otel_config(path);

    init_logs_opentelemetry(&config)
}

/// Initialize the logger with the provided logstash config
#[cfg(feature = "otel")]
pub fn init_logs_opentelemetry(
    config: &OpentelemetryConfig,
) -> AnyResult<opentelemetry_sdk::logs::Logger, LogError> {
    let log_config: Vec<KeyValue> = config
        .log_config
        .iter()
        .map(|(k, v)| KeyValue::new(Key::new(k.clone()), v.to_string()))
        .collect();
    trace!("OLTP log_config: {:?}", log_config);

    let logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(Config::default().with_resource(Resource::new(log_config)))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.endpoint),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to initialize opentelemetry logging");

    // Retrieve the global LoggerProvider.
    let logger_provider = logger_provider();

    // Create a new OpenTelemetryLogBridge using the above LoggerProvider.
    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
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
pub fn init_tracer( config: &OpentelemetryConfig) -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-trace-example",
        )])),)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(config.traces_endpoint.clone()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

// TODO add configuration
#[cfg(feature = "otel")]
pub fn init_metrics(config: &OpentelemetryConfig) -> metrics::Result<MeterProvider> {
    let export_config = opentelemetry_otlp::ExportConfig {
        endpoint: config.metrics_endpoint.to_string(),
        protocol: Protocol::Grpc,
        timeout: Default::default(),
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-metrics-example",
        )]))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .build()
}

/// Returns the OpentelemetryConfig struct converted from the otel config json
pub fn get_otel_config(path: &String) -> OpentelemetryConfig {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let config: OpentelemetryConfig = serde_json::from_str(&contents).unwrap();
    config
}
