# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/brytelands/sologger/compare/sologger_log_transport-v0.4.0...sologger_log_transport-v0.5.0) - 2026-11-11

### Breaking

- OpenTelemetry crates upgraded from 0.32 to 0.33. The public API returns and accepts their types (`SdkLoggerProvider`, `SdkTracerProvider`, `SdkMeterProvider`), so callers must use OpenTelemetry 0.33 too.
- `init_logstash_logger` now returns an error for a missing or invalid log4rs config instead of `Ok(())`.

### Added

- `fanout::FanoutLogger` and `LevelFiltered`, plus `build_logstash_logger` and `build_logs_opentelemetry`, to run the LogStash and OpenTelemetry transports in one process.
- `OtelProviders::shutdown` to deliver buffered logs, spans and metrics before exit.
- `load_otel_config`, which returns errors instead of panicking; `metricsEndpoint`, `tracesEndpoint` and `logConfig` are now optional.

### Other

- V0.4.0 ([#34](https://github.com/brytelands/sologger/pull/34))
