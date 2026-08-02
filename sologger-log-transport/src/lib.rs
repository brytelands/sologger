//!# sologger-log-transport
//!
//!**Overview**
//!
//!This is a library that provides support for both LogStash and OpenTelemetry exports for logs.
//!
//!**Usage**
//!
//!The two features available in this crate are logstash and otel. Both are possible to use at the same time, but it is not recommended. Specifying either logstash or otel is suggested, depending on your needs.
//!
//!**LogStash**
//!
//!```rust
//!init_logstash_logger(&"./tests/configs/logstash_config.json".to_string());
//!```
//!
//!Logstash support is provided by [logstash-rs and log4rs-logstash](https://github.com/qoollo/rust-log4rs-logstash). The logstash appender utilizes log4rs which also
//!provides the ability to log to files and stdout, which provider further flexibility. More information on log4rs configuration can be found here: https://docs.rs/log4rs/latest/src/log4rs/config/raw.rs.html
//!
//!**OpenTelemetry**
//!
//!```rust
//!init_logs_opentelemetry_with_config_path(&"./tests/configs/opentelemetry-config.json".to_string());
//!```
//!
//!OpenTelemetry support is provided by [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust). Logs, traces and metrics are sent to the configured endpoints, such as [Signoz](https://signoz.io/) or [Vector](https://vector.dev/).
//!For a list of all available configuration options, see the [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/otel/resource/semantic_conventions/)
//!
//!The `solana_telemetry` module turns parsed `LogContext` records into one trace per
//!transaction (a span per program invocation, parented by CPI depth) and into metrics
//!(compute-unit histograms, transaction failure / truncated-log / reconnect counters).
//!Enable them with `enableTraces` / `enableMetrics` in the OpenTelemetry config. Span
//!durations are synthetic — consumed compute units rendered as microseconds — because
//!Solana logs carry no timestamps.

#[cfg(feature = "logstash")]
pub mod logstash_lib;
#[cfg(feature = "otel")]
pub mod opentelemetry_config;
#[cfg(feature = "otel")]
pub mod opentelemetry_lib;
#[cfg(feature = "otel")]
pub mod solana_telemetry;
