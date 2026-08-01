use std::fs;
use std::str::FromStr;

use anyhow::Result as AnyResult;
use log::{Level, trace};
use opentelemetry::{Key, KeyValue, global};
use opentelemetry_otlp::{ExporterBuildError, LogExporter, WithExportConfig};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout::SpanExporter;

use crate::opentelemetry_config::OpentelemetryConfig;

/// Initialize the logger with the provided logstash config location
pub fn init_logs_opentelemetry_with_config_path(
    path: &String,
) -> AnyResult<SdkLoggerProvider, ExporterBuildError> {
    let config = get_otel_config(path);

    init_logs_opentelemetry(&config)
}

/// Initialize the logger with the provided logstash config
#[cfg(feature = "otel")]
pub fn init_logs_opentelemetry(
    config: &OpentelemetryConfig,
) -> AnyResult<SdkLoggerProvider, ExporterBuildError> {
    let log_config: Vec<KeyValue> = config
        .log_config
        .iter()
        .map(|(k, v)| KeyValue::new(Key::new(k.clone()), v.to_string()))
        .collect();
    trace!("OLTP log_config: {:?}", log_config);

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(&config.endpoint)
        .build()?;

    let logger = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_attributes(log_config).build())
        .with_batch_exporter(exporter)
        .build();

    // Create a new OpenTelemetryLogBridge using the above SdkLoggerProvider.
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
pub fn init_tracer(_config: &OpentelemetryConfig) -> AnyResult<SdkTracerProvider> {
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();

    Ok(provider)
}

// TODO add configuration
#[cfg(feature = "otel")]
pub fn init_metrics(_config: &OpentelemetryConfig) -> AnyResult<SdkMeterProvider> {
    let exporter = opentelemetry_stdout::MetricExporter::default();
    let reader = PeriodicReader::builder(exporter).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(
            Resource::builder()
                .with_attributes([KeyValue::new("service.name", "metrics-basic-example")])
                .build(),
        )
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
