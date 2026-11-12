# AGENTS.md — sologger

Guide for AI agents working in this repository. Keep it accurate; correct it when you discover drift.

## What this is

sologger parses raw log strings emitted by Solana RPCs into structured `LogContext` records and ships them to LogStash or OpenTelemetry. It is a Cargo workspace of six Rust crates plus a Vue 3 web UI.

```
sologger-log-context/          Core parser. Raw log lines -> LogContext. No Solana deps.
sologger-idl-decoder/          Anchor IDL-driven enrichment: borsh-decodes `Program data:` events
                               into LogContext.decoded_events, maps error_code -> error_name.
                               Supports legacy and 0.30+ IDL specs; serde only, no anchor-lang.
sologger-log-transformer/      Extracts logs out of Solana API types (blocks, txs, RPC responses).
sologger-log-transformer-wasm/ wasm-bindgen wrapper around log-context + idl-decoder for the browser.
sologger-log-transport/        LogStash (log4rs), OpenTelemetry (OTLP) and webhook exporters.
                               Ships logs, opt-in per-transaction CPI span trees + CU/failure
                               metrics (solana_telemetry, via enableTraces/enableMetrics), and
                               rule-filtered Discord/Slack/JSON webhooks (webhook feature).
                               Depends on sologger-log-context (otel + webhook features).
sologger/                      The binary. Subscribes to an RPC over WebSocket (logsSubscribe or
                               blockSubscribe) with supervised auto-reconnect + slot-gap
                               detection, parses, enriches (optional `idls` map), refetches
                               truncated logs, exports. Optional historical backfill mode
                               (`backfill` block) replays past transactions via getTransaction.
sologger-ui/                   Vue 3 + Vite app. Uses the WASM build. See sologger-ui/AGENTS.md.
config/                        Example config pairs, one directory per deployment scenario.
docker-examples/               Compose stacks: Parseable, OpenSearch, ELK, SigNoz.
```

Parsing entry point: `sologger-log-context/src/sologger_log_context.rs`. A single large `LOG_REGEX`
with named capture groups (`programInvoke`, `programLog`, `programData`, `programConsumed`, …) drives
everything. Adding support for a new Solana log line means extending that regex and the match arms
that consume it.

## The single most important gotcha

**Workspace members depend on each other with `{ version = "x", path = "../..." }`, and releases
are automated — do not bump versions or `cargo publish` by hand.**

The path makes local builds and tests use the sibling source; `cargo publish` strips it and keeps
the version requirement. The path is also what tells cargo and release-plz the publish order.
(An earlier layout used bare versions plus a root `[patch.crates-io]`; that made
`cargo publish --workspace` order crates alphabetically, so it was removed — do not bring it back.)

Version bumps, the propagated requirements, `Cargo.lock` and per-crate `CHANGELOG.md` files come
from the release-plz PR; merging it publishes. See `release-plz.toml` and
`.claude/skills/release-crates/`.

## Feature flags

Three independent transports, and the flag names differ depending on which crate you address:

| Crate | Flag | Effect |
|---|---|---|
| `sologger` | `enable_logstash` | turns on `sologger_log_transport/logstash` |
| `sologger` | `enable_otel` | turns on `sologger_log_transport/otel` |
| `sologger` | `enable_webhook` | turns on `sologger_log_transport/webhook` |
| `sologger` | `solana_client_subscriber` | **default on**; pulls in the Solana RPC/pubsub client. The binary (and the tests that launch it) require it; with `--no-default-features` only the library builds, which CI's clippy job checks |
| `sologger` | `enable_tokio_rt_metrics` | tokio runtime metrics |
| `sologger_log_transport` | `logstash`, `otel`, `webhook` | the underlying feature names |

With no transport feature at all (or when a compiled-in transport's config file is absent), the
binary falls back to pretty console output — colored, CPI-indented — rather than panicking. See
`sologger/src/console_logger.rs`.

When building the whole workspace you need both spellings, because `--workspace` resolves features
across every member. That is why CI uses `--features 'enable_logstash enable_otel logstash otel enable_webhook webhook'`.

Both transports can run in one process. The `log` crate allows a single global logger, so each
transport *builds* its logger (`build_logstash_logger`, `build_logs_opentelemetry`) and
`logger_lib::init_logger` installs them together behind `sologger_log_transport::fanout::FanoutLogger`,
each keeping its own level. Do not call `init_logstash_logger` / `init_logs_opentelemetry` from the
binary — those install a global logger and are for library users with a single transport. log4rs
`refresh_rate` hot reload is done by `LogstashLogger::watch_config`, which restores the combined max
level after each reload (log4rs resets it to its own). `sologger/tests/combined_transports_test.rs`
covers the combination, which is why CI's test features include `enable_otel`.

`sologger/src/lib.rs` uses `#[path = "solana_client_subscriber.rs"]` (only with the
`solana_client_subscriber` feature) to map that file onto the module name `log_subscriber`. If you
grep for `log_subscriber` and find no such file, that is why.

## Commands

Verified working as of this writing (cargo 1.97.1, node 24.12.0, wasm-pack 0.14.0).

```bash
# Fast feedback — matches the feature set CI tests with
cargo check --workspace --features 'enable_logstash enable_otel logstash otel enable_webhook webhook'
cargo test  --workspace --features 'enable_logstash enable_otel logstash otel enable_webhook webhook'

# What CI actually runs
cargo build --locked --release --features 'enable_logstash enable_otel enable_webhook'
cargo fmt --all --check
cargo clippy --locked --all --features 'enable_logstash enable_otel enable_webhook' -- -D warnings
cargo llvm-cov --locked --features 'enable_logstash enable_otel logstash otel enable_webhook webhook' --workspace --lcov --output-path lcov.info

# Benchmarks (criterion)
cargo bench -p sologger_log_context
cargo bench -p sologger_log_transformer

# WASM
cd sologger-log-transformer-wasm && wasm-pack build --target web

# UI — note there is no `npm test` script defined
cd sologger-ui && npm install && npm run dev
cd sologger-ui && npx vitest run
cd sologger-ui && npm run test:e2e   # Playwright; first run `npx playwright install chromium`
```

## Running the binary

Config resolution order in `sologger/src/main.rs`:

1. first CLI argument, else
2. `SOLOGGER_APP_CONFIG_LOC` env var, else
3. `./config/local/sologger-config.json`

The variable is `SOLOGGER_APP_CONFIG_LOC` — the short form — everywhere: code, READMEs,
Dockerfiles, and compose files all agree. (The READMEs once said `SOLOGGER_APP_CONFIG_LOCATION`;
that drift has been fixed.)

Configs come in *pairs* — a `sologger-config.json` plus the transport config it points at:

| Directory | Pairs with | For |
|---|---|---|
| `config/local/` | `log4rs-config.yml` | local dev, devnet, LogStash build |
| `config/demo/` | `log4rs-config.yml` | demo |
| `config/docker/` | both yml + otel json | container runs |
| `config/logstash-example/` | `log4rs-config.yml` | LogStash reference |
| `config/otel-example/` | `opentelemetry-config.json` | OTel reference |
| `config/file-example/` | `log4rs-config.yml` | file sink |
| `config/webhook-example/` | `webhook-config.json` | webhook reference |
| `config/combined-example/` | logstash-example's yml + otel-example's json | both transports in one binary |

`SologgerConfig` (`sologger/src/sologger_config.rs`) is `#[serde(rename_all = "camelCase")]`, so JSON
keys are `log4rsConfigLocation`, `opentelemetryConfigLocation`, `rpcUrl`, `rpcHttpUrl`, `source`, `backfillTruncated`,
`backfill`, `commitmentLevel`, `allWithVotes`, `webhookConfigLocation`. `programsSelector` is
*not* part of that struct — it is parsed separately in `main.rs` into a `ProgramsSelector`. A
`programs` array of `["*"]` means select-all. Paths inside a config are relative to the working
directory you launch from, which is assumed to be the repo root.

The binary runs until SIGINT/SIGTERM and never reads stdin, so it works detached (Docker without
`-i`/`-t`, systemd, nohup). It used to wait on stdin and died at startup with "Error starting
WebSocket for log subscription: unexpected end of file" whenever stdin was closed — that message
did not mean a network problem. `sologger/tests/lifecycle_test.rs` guards this.

On SIGINT/SIGTERM, `logger_lib::shutdown_transports` shuts down the OpenTelemetry providers before
exiting, so batched logs/spans and metrics since the last export (default interval 60s) are
delivered instead of dropped. It gives up after 5s (`SHUTDOWN_TIMEOUT` in `main.rs`) when the
collector is unreachable. `OpenTelemetryLogBridge::flush()` is a no-op — flushing the `log` logger
alone does not deliver anything. Covered by `sologger-log-transport/tests/otel_shutdown_tests.rs`.

To test OTel end to end, run `otel/opentelemetry-collector-contrib` with an OTLP gRPC receiver
on 4317 and `file` exporters for logs/traces/metrics, point `endpoint` at it, and set
`OTEL_METRIC_EXPORT_INTERVAL=10000` to see periodic metric exports during a short run. After a
SIGTERM, the last `sologger.transactions` value should equal the number of root `transaction` spans.

A transport whose config file is missing or invalid (unparseable log4rs config; OpenTelemetry
config that is not valid JSON or lacks `endpoint`/`logLevel`; unusable OTLP endpoint) is *disabled
with a notice* naming the file and problem, and when no transport ends up active the binary falls
back to pretty console output (it used to panic). In the OpenTelemetry config, `metricsEndpoint`,
`tracesEndpoint` and `logConfig` are optional. The binary uses `load_otel_config`, which returns
errors; `get_otel_config` still panics and exists for library compatibility. A
`webhookConfigLocation` that is set but unreadable or invalid disables the webhook the same way
(`sologger/tests/webhook_config_test.rs`).

`log_processor::log_log_context` attaches each `LogContext`'s fields to its `log` record as
key-values (`solana.signature`, `solana.slot`, `solana.program_id`, …, the same names as the span
attributes in `solana_telemetry`). The OpenTelemetry bridge exports them as log record
attributes; the logstash appender ignores them. This needs `log`'s `kv` feature, which
`sologger/Cargo.toml` enables.

## Generated artifacts and deployment

The public site <https://brytelands.github.io/sologger/> is deployed by
`.github/workflows/pages.yml`: on every push to `main` touching the UI or the parser crates, it
builds the WASM transformer **from current source**, overlays it into the UI, runs the UI tests,
builds with Vite, and publishes `dist/` via `actions/deploy-pages`. The Pages source in the repo
settings must be set to "GitHub Actions" for this to serve. There is no tracked copy of the built
site. The workflow copies `index.html` to `404.html`: Pages serves that for any path it has no file
for, which is how deep links such as `/sologger/lookup?sig=…` reach the app's router (with HTTP
status 404). Without it they get GitHub's "Page not found".

One committed artifact remains, deliberately:

```
sologger-log-transformer-wasm/pkg/            wasm-pack output (gitignored)
  -> sologger-ui/public/sologger-log-transformer-wasm/pkg/   (tracked)
```

The tracked `public/.../pkg/` copy exists because `HomeView.vue` and `ConvertView.vue` import it
directly — without it, `npm run dev` and the `ui.yml` CI build fail on a fresh clone. It can lag the
Rust source between syncs (run `.claude/skills/wasm-sync/` after parser changes); the deployed site
never lags, because `pages.yml` rebuilds and overwrites it during every deploy.

Vite `base` is `/sologger/`, matched by the router's `createWebHistory('/sologger/')` — changing one
without the other breaks deep links on Pages.

Docker images (linux/amd64 + linux/arm64) are built on pull requests that touch them and pushed to
Docker Hub only for a `v*` release tag, which must match the version in `sologger/Cargo.toml`
(`.github/workflows/docker.yml`). A sologger release runs it from `release-plz.yml`, together with
`release-binaries.yml` (Linux x86_64/arm64 archives + `SHA256SUMS` on the GitHub Release).
Both check out `refs/tags/<tag>`, never the bare tag name: release tags (`v0.4.1`) share their
names with integration branches, and `actions/checkout` given a bare name picks the branch. That
built the 0.4.0 artifacts from the `v0.4.0` branch and failed the 0.4.1 ones (version mismatch).

The compose stacks (`docker-compose.yaml`, `docker-examples/*`) do not use those published images:
each sologger service builds from the repo (`build.context` is the repo root, relative to the compose
file) as `sologger-logstash:local` / `sologger-otel:local`, with `pull_policy: build`. Keep
`.dockerignore` excluding `/target` and `node_modules` — the whole repo root is the build context.

The root `docker-compose.yaml` also has a `sologger-ui` service: `sologger-ui/Dockerfile` builds the
site with Node and serves `dist/` from unprivileged nginx at `http://localhost:8080/sologger/`
(`sologger-ui/nginx.conf` falls back to `index.html` for client routes). Its build context is
`sologger-ui/` alone, with its own `.dockerignore`. It uses the committed WASM in `public/`, so it
lags parser changes until that is re-synced. Not published to Docker Hub.

CI (`rust.yml`, `ui.yml`, `audit.yml`) runs on pull requests into `main` and into `v*`
integration branches such as `v0.4.0`.

`Cargo.lock` is tracked (this workspace produces a binary). Commit lockfile changes alongside the
dependency change that caused them; do not regenerate it gratuitously.

## Known baseline noise

Do not treat these as regressions you caused:

- Test builds of `sologger-log-transformer` emit a few unused-import warnings
  (`from_confirmed_block`, `ConfirmedBlock`, `InstructionError`). The library builds themselves are
  warning-free — `sologger-log-transport` is on OpenTelemetry 0.33 and must stay warning-free.
- `cargo audit` (run weekly and on dependency changes by `.github/workflows/audit.yml`) passes with
  a handful of *warnings* — unmaintained/unsound advisories against crates pinned deep in the Solana
  dependency tree (`bincode`, `rand`). Warnings do not fail the job; new *vulnerabilities* do, and
  are usually fixable with a targeted `cargo update -p <crate>`. Accepted exceptions live in
  `.cargo/audit.toml`, each with its reason.
- `sologger-log-transformer` allows `deprecated` in `log_context_transformer.rs` because the Solana
  3.x status types are flagged ahead of 4.0. Drop that when upgrading the Solana crates.

## Conventions

- Rust 2021 edition, no `rust-toolchain.toml` — CI builds on `dtolnay/rust-toolchain@stable`. MSRV is
  **1.92.0**, declared once as `[workspace.package] rust-version` in the root `Cargo.toml` and
  inherited by every member; the CI `msrv` job and the Dockerfiles' `ARG RUST_VERSION` both pin it.
  Bump all three together or not at all.
- All library crates set `doctest = false`. Doc comments contain illustrative, non-compiling
  snippets; do not add doctests expecting them to run.
- Tests live inline in `#[cfg(test)] mod tests` for most crates; `sologger/tests/` and
  `sologger-log-transport/tests/` hold the integration tests.
- Public items on the parser types carry `///` docs — match that density when adding fields to
  `LogContext`.
- Crate versions are not in lockstep; release-plz bumps only crates that changed. Tags are
  `<crate>-v<version>` for libraries and `v<version>` for the `sologger` binary.

## Scope notes

- `sologger-ui/.junie/agent.md` is a JetBrains Junie guide for the same UI. It is partly stale (it
  describes Handsontable; the UI moved to SlickGrid). `sologger-ui/AGENTS.md` is the current one.
- `docs/tasks.md` and `docs/future_enhancments.md` under `sologger-ui/` are a completed feature
  backlog, useful as intent history rather than as instructions.
