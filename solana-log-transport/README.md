# solana-log-transport

**Overview**

This is a library that provides support for both LogStash and OpenTelemetry exports for logs.

**Usage**

The two features available in this crate are logstash and otel. Both are possible to use at the same time, but it is not recommended. Specifying either logstash or otel is suggested, depending on your needs.

**LogStash**

```rust
init_logstash_logger(&"./tests/configs/logstash_config.json".to_string());
```

Logstash support is provided by [logstash-rs and log4rs-logstash](https://github.com/qoollo/rust-log4rs-logstash). The logstash appender utilizes log4rs which also
provides the ability to log to files and stdout, which provider further flexibility. More information on log4rs configuration can be found here: https://docs.rs/log4rs/latest/src/log4rs/config/raw.rs.html

**OpenTelemetry**

```rust
init_logs_opentelemetry_with_config_path(&"./tests/configs/opentelemetry-config.json".to_string());
```

OpenTelemetry support is provided by [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust). Currently, tracer and metrics functionality are not supported. Using the OpenTelemetry exporter will result in logs being sent to the configured endpoint, such as [Signoz](https://signoz.io/) or [Vector](https://vector.dev/)
For a list of all available configuration options, see the [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/otel/resource/semantic_conventions/)