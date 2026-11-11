#![cfg(feature = "otel")]
//! Loading the OpenTelemetry config reports problems as errors that name the file and
//! the problem, instead of panicking.

use std::path::PathBuf;

use sologger_log_transport::opentelemetry_lib::load_otel_config;

fn write_config(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sologger-otel-config-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

fn error_for(path: &PathBuf) -> String {
    format!(
        "{:#}",
        load_otel_config(path.to_str().unwrap()).unwrap_err()
    )
}

#[test]
fn a_missing_required_field_is_an_error_naming_it() {
    let path = write_config("no-endpoint.json", r#"{ "logLevel": "INFO" }"#);
    let error = error_for(&path);
    assert!(error.contains("endpoint"), "{error}");
    assert!(error.contains("no-endpoint.json"), "{error}");
}

#[test]
fn invalid_json_is_an_error() {
    let path = write_config("broken.json", r#"{ "endpoint": "#);
    let error = error_for(&path);
    assert!(error.contains("broken.json"), "{error}");
}

#[test]
fn a_missing_file_is_an_error() {
    let error = format!(
        "{:#}",
        load_otel_config("./tests/configs/does-not-exist.json").unwrap_err()
    );
    assert!(error.contains("does-not-exist.json"), "{error}");
}

#[test]
fn optional_fields_default_to_empty() {
    let path = write_config(
        "minimal.json",
        r#"{ "endpoint": "http://localhost:4317", "logLevel": "INFO" }"#,
    );
    let config = load_otel_config(path.to_str().unwrap()).unwrap();

    assert_eq!(config.endpoint, "http://localhost:4317");
    // empty signal endpoints fall back to `endpoint`
    assert_eq!(config.metrics_endpoint, "");
    assert_eq!(config.traces_endpoint, "");
    assert!(config.log_config.is_empty());
    assert!(!config.enable_traces && !config.enable_metrics);
}

#[test]
fn the_example_configs_still_load() {
    load_otel_config("./tests/configs/opentelemetry-config.json").unwrap();
    load_otel_config("../config/otel-example/opentelemetry-config.json").unwrap();
    load_otel_config("../config/docker/opentelemetry-config.json").unwrap();
}
