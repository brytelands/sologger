use opentelemetry::logs::LoggerProvider;
use sologger_log_transport::opentelemetry_lib::{get_otel_config, init_logs_opentelemetry_with_config_path, init_metrics, init_tracer};

#[tokio::test]
pub async fn test_load_config() {
    let result = init_logs_opentelemetry_with_config_path(&"./tests/configs/opentelemetry-config.json".to_string());
    assert!(result.is_ok());
}


#[tokio::test]
pub async fn test_init_trace() {
    let otel_config = get_otel_config(&"./tests/configs/opentelemetry-config.json".to_string());
    let result = init_tracer(&otel_config);
    assert!(result.is_ok());
}

#[tokio::test]
pub async fn test_init_metrics() {
    let otel_config = get_otel_config(&"./tests/configs/opentelemetry-config.json".to_string());
    let result = init_metrics(&otel_config);
    assert!(result.is_ok());
}
