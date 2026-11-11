# sologger-log-transport

**Overview**

This is a library that ships sologger's structured logs over LogStash (via log4rs) and OpenTelemetry (OTLP), exports
per-transaction traces and metrics over OpenTelemetry, and posts rule-filtered webhooks.

**Usage**

Three features, usable together: `logstash`, `otel` and `webhook`.

The `log` crate allows a single global logger. With one transport, the `init_*` functions build it and install it.
To run LogStash and OpenTelemetry together, build each logger with `build_logstash_logger` /
`build_logs_opentelemetry` and install them together with `fanout::FanoutLogger`; each keeps its own level.

**LogStash**

```rust
init_logstash_logger("./tests/configs/log4rs-config.yml")?;
```

A missing or invalid config is an error. When the log4rs config sets `refresh_rate`, the file is reloaded when it
changes, like log4rs's own `init_file`.

Logstash support is provided by [logstash-rs and log4rs-logstash](https://github.com/qoollo/rust-log4rs-logstash). The logstash appender utilizes log4rs which also
provides the ability to log to files and stdout, which provider further flexibility. More information on log4rs configuration can be found here: https://docs.rs/log4rs/latest/src/log4rs/config/raw.rs.html

**OpenTelemetry**

```rust
let config = get_otel_config(&"./tests/configs/opentelemetry-config.json".to_string());
let providers = OtelProviders {
    logs: Some(init_logs_opentelemetry(&config)?),
    traces: Some(init_tracer(&config)?),   // when exporting traces
    metrics: Some(init_metrics(&config)?), // when exporting metrics
};

// ... before the process exits:
providers.shutdown();
```

Logs and spans are batched and metrics are exported periodically, so keep the providers and call
`OtelProviders::shutdown` before exiting; otherwise whatever is still buffered is lost. `solana_telemetry` turns parsed
`LogContext` records into one trace per transaction (a span per program invocation) and compute-unit/failure metrics.

OpenTelemetry support is provided by [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust). Logs, traces
and metrics are sent over OTLP/gRPC to the configured endpoints, such as an OpenTelemetry Collector,
[SigNoz](https://signoz.io/) or [Vector](https://vector.dev/).
For a list of all available configuration options, see the [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/otel/resource/semantic_conventions/)

**LogStash and OpenTelemetry together**

```rust
let logstash = build_logstash_logger("./log4rs-config.yml")?;
let (log_provider, otel_bridge) = build_logs_opentelemetry(&otel_config)?;

let handle = logstash.handle();
let otel_level = otel_bridge.level();
// log4rs resets the global max level on reload; keep OpenTelemetry's level available too
logstash.watch_config(move || log::set_max_level(handle.max_log_level().max(otel_level)));

log::set_max_level(logstash.max_level().max(otel_level));
log::set_boxed_logger(Box::new(FanoutLogger::new(vec![
    Box::new(logstash.into_logger()),
    Box::new(otel_bridge),
])))?;
```
