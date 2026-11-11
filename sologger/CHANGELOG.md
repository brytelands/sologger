# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/brytelands/sologger/releases/tag/v0.4.0) - 2026-11-11

### Other

- V0.4.0 ([#34](https://github.com/brytelands/sologger/pull/34))
- Bump sologger to 0.3.0, sologger_log_transformer and sologger_log_transformer_wasm to 0.3.0 for the phase 1-6 release
- Add historical backfill, truncation backfill, and webhook transport support; refactor configuration and logging initialization for robust ingestion
- Bump `sologger_log_transport` to 0.4.0, add trace and metric support in OpenTelemetry configuration, and refactor initialization logic to enable optional telemetry features.
- Add `decode_program_data` utility and support for IDL-based event decoding
- Bump `sologger_log_context` to 0.3.0, extend log parsing to support instruction names and error codes, and update dependencies and tests across packages.
- Bump sologger_log_transport to 0.3.0 and update OpenTelemetry dependencies to 0.32.0, refactoring log, trace, and metrics initialization to align with the updated API.
- Add MSRV (1.92.0) workflow and enforce rust-version in workspace
- Update workflows, UI pipeline, logging, and configuration references
- Bump version to 0.2.4 and update Solana-related dependencies and sologger_log_context to 0.2.4
- Upgrade Sologger UI dependencies to the latest versions for compatibility and stability improvements.
- Upgraded various dependencies, including `log`, `serde`, `tokio`, and Solana-related crates, to their latest versions for stability and compatibility improvements. Added support for the `cost_units` field in the `log_context_transformer` module.
- 8 dont detected event in cpi ([#9](https://github.com/brytelands/sologger/pull/9))
- Upgrade dependencies and fix wasm import paths
- Add consumed_cu and max_cu fields to LogContext struct
- V0.3.0 ([#6](https://github.com/brytelands/sologger/pull/6))
- Update sologger_log_transformer to version 0.2.1
- Update version to 0.2.1 and improve descriptions
- Update Dockerfile bindings and add conditional otel import
- Update version to 0.2.0 and use external crate dependencies
- Update dependencies and refine OpenTelemetry integration
- Update Solana dependency to 1.18.12
- Dependency update - Solana 1.18
- Dependency updates - solana, sologger libs
- Dependency updates - solana, sologger libs
- Updated solana dependency versions to 1.16.20
- Updated solana dependency versions to 1.16.20
- Updated sologger_log_context dep to initial release
- Updated license in toml
- Renamed libraries prefixed with solana to sologger
- Introducing sologger!
