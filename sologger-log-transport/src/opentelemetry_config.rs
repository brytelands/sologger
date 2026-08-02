use opentelemetry::{Key, KeyValue};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;
use std::collections::HashMap;

/// OpenTelemetry configuration
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpentelemetryConfig {
    ///OpenTelemetry configuration as Key/Value pairs. For a list of configuration options, see the [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/otel/resource/semantic_conventions/)
    pub log_config: HashMap<String, String>,
    ///The OTel endpoint to send logs to, used by opentelemetry_otlp
    pub endpoint: String,
    ///The OTel endpoint to send metrics to
    pub metrics_endpoint: String,
    ///The OTel endpoint to send traces to
    pub traces_endpoint: String,
    ///Sets the global maximum log level.
    pub log_level: String,
    ///Export each transaction as a trace: one span per program invocation, parented by
    ///CPI depth. Span durations are synthetic (proportional to consumed compute units),
    ///since Solana logs carry no timestamps. Off by default.
    #[serde(default)]
    pub enable_traces: bool,
    ///Export metrics: compute-unit histograms per program/instruction, transaction
    ///failure counter, truncated-log counter, WebSocket reconnect counter. Off by default.
    #[serde(default)]
    pub enable_metrics: bool,
}

/// OpenTelemetry configuration
impl OpentelemetryConfig {
    /// Converts the log_config map into a Vec of OTel API KeyValues
    pub fn key_values(&self) -> Vec<KeyValue> {
        self.log_config
            .iter()
            .map(|(k, v)| KeyValue::new(Key::new(k.clone()), v.to_string()))
            .collect()
    }
}

#[test]
pub fn test_deserialize_all() {
    let config = json!(
        {
          "logConfig": {
            "service.name": "basic-otlp-logging-example"
          },
          "endpoint": "http://localhost:4317",
          "metricsEndpoint": "http://localhost:4318/v1/metrics",
          "tracesEndpoint": "http://localhost:4318/v1/traces",
          "logLevel": "info"
        }
    );

    let config = serde_json::from_value::<OpentelemetryConfig>(config).unwrap();
    assert_eq!(config.endpoint, "http://localhost:4317");
    assert_eq!(config.metrics_endpoint, "http://localhost:4318/v1/metrics");
    assert_eq!(config.traces_endpoint, "http://localhost:4318/v1/traces");
    assert_eq!(
        config.log_config.get("service.name").unwrap(),
        "basic-otlp-logging-example"
    );
}

#[test]
pub fn test_key_value() {
    let config = json!(
        {
          "logConfig": {
            "service.name": "basic-otlp-logging-example"
          },
          "endpoint": "http://localhost:4317",
          "metricsEndpoint": "http://localhost:4318/v1/metrics",
          "tracesEndpoint": "http://localhost:4318/v1/traces",
          "logLevel": "info"
        }
    );

    let config = serde_json::from_value::<OpentelemetryConfig>(config).unwrap();
    let key_values = config.key_values();

    let key_value = key_values.get(0).unwrap();
    assert_eq!(key_value.key.as_str(), "service.name");
    assert_eq!(key_value.value.as_str(), "basic-otlp-logging-example");
}
