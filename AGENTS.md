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

**Workspace members depend on each other by published version, redirected locally via a patch.**

`sologger/Cargo.toml` says `sologger_log_context = "0.2.4"`, not `{ path = ... }` — the manifests
publish cleanly to crates.io that way. A `[patch.crates-io]` section in the root `Cargo.toml`
redirects those requirements to the sibling directories, so local builds and tests exercise the code
in this repo.

Two consequences:

- **Do not remove that patch section.** Without it, every crate silently resolves to the crates.io
  copies and local edits to the libraries stop reaching the binary and the WASM bundle.
- **Version requirements still matter.** The patch only applies while each local crate's version
  satisfies the dependent's requirement. When bumping a library's version, propagate the new
  requirement into its dependents' manifests in the same change (see
  `.claude/skills/release-crates/`), or the patch stops matching and resolution falls back to the
  registry.

Publishing is unaffected: patches are not included in packaged crates.

## Feature flags

Three independent transports, and the flag names differ depending on which crate you address:

| Crate | Flag | Effect |
|---|---|---|
| `sologger` | `enable_logstash` | turns on `sologger_log_transport/logstash` |
| `sologger` | `enable_otel` | turns on `sologger_log_transport/otel` |
| `sologger` | `enable_webhook` | turns on `sologger_log_transport/webhook` |
| `sologger` | `solana_client_subscriber` | **default on**; pulls in the Solana RPC/pubsub client |
| `sologger` | `enable_tokio_rt_metrics` | tokio runtime metrics |
| `sologger_log_transport` | `logstash`, `otel`, `webhook` | the underlying feature names |

With no transport feature at all (or when a compiled-in transport's config file is absent), the
binary falls back to pretty console output — colored, CPI-indented — rather than panicking. See
`sologger/src/console_logger.rs`.

When building the whole workspace you need both spellings, because `--workspace` resolves features
across every member. That is why CI uses `--features 'enable_logstash logstash otel'`.

Building with both transports at once compiles, but the README advises against running it that way —
ship two binaries instead.

`sologger/src/lib.rs` uses `#[cfg_attr(feature = "solana_client_subscriber", path = "...")]` to map
`solana_client_subscriber.rs` onto the module name `log_subscriber`. If you grep for `log_subscriber`
and find no such file, that is why.

## Commands

Verified working as of this writing (cargo 1.97.1, node 24.12.0, wasm-pack 0.14.0).

```bash
# Fast feedback — matches the feature set CI tests with
cargo check --workspace --features 'enable_logstash logstash otel'
cargo test  --workspace --features 'enable_logstash logstash otel'

# What CI actually runs
cargo build --release --features 'enable_logstash enable_otel'
cargo clippy --all --features 'enable_logstash enable_otel'
cargo llvm-cov --features 'enable_logstash logstash otel' --workspace --lcov --output-path lcov.info

# Benchmarks (criterion)
cargo bench -p sologger_log_context
cargo bench -p sologger_log_transformer

# WASM
cd sologger-log-transformer-wasm && wasm-pack build --target web

# UI — note there is no `npm test` script defined
cd sologger-ui && npm install && npm run dev
cd sologger-ui && npx vitest run
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

`SologgerConfig` (`sologger/src/sologger_config.rs`) is `#[serde(rename_all = "camelCase")]`, so JSON
keys are `log4rsConfigLocation`, `rpcUrl`, `rpcHttpUrl`, `source`, `backfillTruncated`,
`backfill`, `commitmentLevel`, `allWithVotes`, `webhookConfigLocation`. `programsSelector` is
*not* part of that struct — it is parsed separately in `main.rs` into a `ProgramsSelector`. A
`programs` array of `["*"]` means select-all. Paths inside a config are relative to the working
directory you launch from, which is assumed to be the repo root.

A transport whose config file is missing is *disabled with a notice*, and when no transport ends up
active the binary falls back to pretty console output (it used to panic). The exception is a
`webhookConfigLocation` that is set but unreadable or invalid — that is still a startup panic.

## Generated artifacts and deployment

The public site <https://brytelands.github.io/sologger/> is deployed by
`.github/workflows/pages.yml`: on every push to `main` touching the UI or the parser crates, it
builds the WASM transformer **from current source**, overlays it into the UI, runs the UI tests,
builds with Vite, and publishes `dist/` via `actions/deploy-pages`. The Pages source in the repo
settings must be set to "GitHub Actions" for this to serve. There is no tracked copy of the built
site.

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

`Cargo.lock` is tracked (this workspace produces a binary). Commit lockfile changes alongside the
dependency change that caused them; do not regenerate it gratuitously.

## Known baseline noise

Do not treat these as regressions you caused:

- Test builds of `sologger-log-transformer` emit a few unused-import warnings
  (`from_confirmed_block`, `ConfirmedBlock`, `InstructionError`). The library builds themselves are
  warning-free — `sologger-log-transport` is on OpenTelemetry 0.32 and must stay warning-free.
- `cargo audit` (run weekly and on dependency changes by `.github/workflows/audit.yml`) passes with
  a handful of *warnings* — unmaintained/unsound advisories against crates pinned deep in the Solana
  dependency tree (`bincode`, `rand`, `keccak`). Warnings do not fail the job; new *vulnerabilities*
  do, and are usually fixable with a targeted `cargo update -p <crate>`.

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
- Crate versions are bumped by hand and are not in lockstep. Publishing order matters because of the
  crates.io dependency arrangement above — see `.claude/skills/release-crates/`.

## Scope notes

- `sologger-ui/.junie/agent.md` is a JetBrains Junie guide for the same UI. It is partly stale (it
  describes Handsontable; the UI moved to SlickGrid). `sologger-ui/AGENTS.md` is the current one.
- `docs/tasks.md` and `docs/future_enhancments.md` under `sologger-ui/` are a completed feature
  backlog, useful as intent history rather than as instructions.
