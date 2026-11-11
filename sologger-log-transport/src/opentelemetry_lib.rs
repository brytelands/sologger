use std::fs;
use std::str::FromStr;

use anyhow::{Context, Result as AnyResult};
use log::{trace, Level};
use opentelemetry::{global, Key, KeyValue};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{ExporterBuildError, LogExporter, WithExportConfig};
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_stdout::SpanExporter;

use crate::fanout::LevelFiltered;
use crate::opentelemetry_config::OpentelemetryConfig;

/// Installs the OpenTelemetry log transport from the config file at `path`. Panics on a
/// missing or invalid file, or when a logger is already installed.
pub fn init_logs_opentelemetry_with_config_path(
    path: &str,
) -> AnyResult<SdkLoggerProvider, ExporterBuildError> {
    let config = get_otel_config(path);

    init_logs_opentelemetry(&config)
}

/// The `log` bridge into OpenTelemetry, limited to the config's `logLevel`.
pub type OtelLogBridge = LevelFiltered<OpenTelemetryLogBridge<SdkLoggerProvider, SdkLogger>>;

/// Installs the OpenTelemetry log transport as the process-wide logger. Panics if a
/// logger is already installed — to run alongside other transports, use
/// [`build_logs_opentelemetry`] with a [`crate::fanout::FanoutLogger`] instead.
#[cfg(feature = "otel")]
pub fn init_logs_opentelemetry(
    config: &OpentelemetryConfig,
) -> AnyResult<SdkLoggerProvider, ExporterBuildError> {
    let (logger, bridge) = build_logs_opentelemetry(config)?;
    log::set_max_level(bridge.level());
    log::set_boxed_logger(Box::new(bridge))
        .expect("a logger is already installed; combine transports with FanoutLogger");
    Ok(logger)
}

/// Builds the OTLP log exporter and a `log` bridge to it without installing the bridge,
/// so it can be combined with other transports in a [`crate::fanout::FanoutLogger`].
/// Keep the returned provider and shut it down before exit to deliver buffered logs.
#[cfg(feature = "otel")]
pub fn build_logs_opentelemetry(
    config: &OpentelemetryConfig,
) -> AnyResult<(SdkLoggerProvider, OtelLogBridge), ExporterBuildError> {
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

    let level = Level::from_str(&config.log_level)
        .unwrap_or(Level::Error)
        .to_level_filter();
    let bridge = LevelFiltered::new(OpenTelemetryLogBridge::new(&logger), level);
    Ok((logger, bridge))
}

/// Initializes the tracer provider and installs it globally. Spans export over OTLP
/// (tonic) to `tracesEndpoint` — falling back to `endpoint` — or to stdout when neither
/// is configured, which suits local development.
#[cfg(feature = "otel")]
pub fn init_tracer(config: &OpentelemetryConfig) -> AnyResult<SdkTracerProvider> {
    let resource = Resource::builder()
        .with_attributes(config.key_values())
        .build();
    let builder = SdkTracerProvider::builder().with_resource(resource);

    let endpoint = if config.traces_endpoint.is_empty() {
        &config.endpoint
    } else {
        &config.traces_endpoint
    };
    let provider = if endpoint.is_empty() {
        builder
            .with_simple_exporter(SpanExporter::default())
            .build()
    } else {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        builder.with_batch_exporter(exporter).build()
    };

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

/// Initializes the meter provider and installs it globally. Metrics export over OTLP
/// (tonic) to `metricsEndpoint` — falling back to `endpoint` — or to stdout when neither
/// is configured.
#[cfg(feature = "otel")]
pub fn init_metrics(config: &OpentelemetryConfig) -> AnyResult<SdkMeterProvider> {
    let resource = Resource::builder()
        .with_attributes(config.key_values())
        .build();

    let endpoint = if config.metrics_endpoint.is_empty() {
        &config.endpoint
    } else {
        &config.metrics_endpoint
    };
    // PeriodicReader is generic over the exporter, so each branch builds its own provider
    let provider = if endpoint.is_empty() {
        SdkMeterProvider::builder()
            .with_reader(
                PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build(),
            )
            .with_resource(resource)
            .build()
    } else {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(exporter).build())
            .with_resource(resource)
            .build()
    };
    global::set_meter_provider(provider.clone());
    Ok(provider)
}

/// The providers an application installed. They buffer telemetry — batched logs and
/// spans, and metrics between periodic exports — so call [`OtelProviders::shutdown`]
/// before the process exits, or whatever is still buffered is lost.
#[derive(Clone, Debug, Default)]
pub struct OtelProviders {
    pub logs: Option<SdkLoggerProvider>,
    pub traces: Option<SdkTracerProvider>,
    pub metrics: Option<SdkMeterProvider>,
}

impl OtelProviders {
    /// Exports everything still buffered, then shuts each provider down. Returns one
    /// message per provider that failed; an empty list means everything was delivered.
    pub fn shutdown(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(provider) = &self.logs {
            if let Err(err) = provider.shutdown() {
                errors.push(format!("logs: {err}"));
            }
        }
        if let Some(provider) = &self.traces {
            if let Err(err) = provider.shutdown() {
                errors.push(format!("traces: {err}"));
            }
        }
        if let Some(provider) = &self.metrics {
            if let Err(err) = provider.shutdown() {
                errors.push(format!("metrics: {err}"));
            }
        }
        errors
    }
}

/// Reads the OpenTelemetry config json. A missing or unreadable file, invalid JSON, or a
/// missing required field (`endpoint`, `logLevel`) is an error naming the file and problem.
pub fn load_otel_config(path: &str) -> AnyResult<OpentelemetryConfig, anyhow::Error> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("reading OpenTelemetry config '{path}'"))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("invalid OpenTelemetry config '{path}'"))
}

/// Returns the OpentelemetryConfig struct converted from the otel config json. Panics on
/// a missing or invalid file; [`load_otel_config`] returns the error instead.
pub fn get_otel_config(path: &str) -> OpentelemetryConfig {
    load_otel_config(path).unwrap_or_else(|err| panic!("{err:#}"))
}
