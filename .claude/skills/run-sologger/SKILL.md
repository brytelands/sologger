---
name: run-sologger
description: Build and run the sologger binary or its Docker stacks against a Solana RPC, picking the right transport feature flag and matching config pair. Use when asked to run, start, launch, or manually test sologger, or to reproduce log-parsing behavior end to end.
---

# Run sologger

Getting this running requires three things to agree: the **feature flag** the binary was built with,
the **sologger-config.json** you point it at, and the **transport config** that file references. A
mismatch does not crash: a transport whose config is missing or invalid is disabled with a notice on
stderr, and with no transport left sologger falls back to pretty console output. So "it runs but
prints colored transaction trees" usually means the transport you wanted is not active — read stderr.

## Choose a transport first

| Goal | Feature | Transport config |
|---|---|---|
| Local dev, logs to file/stdout | `enable_logstash` | `log4rs-config.yml` |
| Ship to LogStash / OpenSearch / ELK | `enable_logstash` | `log4rs-config.yml` |
| Ship to an OTLP endpoint (SigNoz, Vector) | `enable_otel` | `opentelemetry-config.json` |
| Both at once | `enable_logstash enable_otel` | both (`config/combined-example/`) |
| Quick look, no backend | none | none — pretty console output |

Both transports can run in one binary; each keeps its own level. `solana_client_subscriber` is on by
default and is what provides the WebSocket subscription.

## Fastest path — local, devnet, file output

`config/local/` is already wired to devnet with a set of native program IDs.

```bash
cargo run --features enable_logstash -- ./config/local/sologger-config.json
```

Run from the **repo root**. Paths inside the config files are relative to the working directory, and
`config/local/sologger-config.json` points at `./config/local/log4rs-config.yml`.

Config resolution, in order (`sologger/src/main.rs`):

1. first CLI argument
2. `SOLOGGER_APP_CONFIG_LOC` environment variable
3. `./config/local/sologger-config.json`

The variable is `SOLOGGER_APP_CONFIG_LOC` — code, READMEs, and every Dockerfile agree on the
short form.

```bash
SOLOGGER_APP_CONFIG_LOC=./config/otel-example/sologger-config.json \
  cargo run --features enable_otel
```

## Point it at different programs

`programsSelector.programs` is parsed separately from the rest of the config, in
`create_programs_selector_from_config`. `["*"]` selects everything; an empty array selects nothing.

```json
{
    "log4rsConfigLocation": "./config/local/log4rs-config.yml",
    "rpcUrl": "wss://api.devnet.solana.com",
    "programsSelector": { "programs": ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"] }
}
```

`rpcUrl` must be a **WebSocket** URL (`wss://`), not `https://` — this is a `logsSubscribe` stream.
Optional keys: `commitmentLevel` (string), `allWithVotes` (bool, default false — vote transactions
are noisy and excluded unless you opt in). Keys are camelCase.

Prefer devnet for testing. Public mainnet RPCs rate-limit `logsSubscribe` aggressively; if the user
wants mainnet, they need a private endpoint.

## Docker

```bash
docker build -f Dockerfile-logstash --tag sologger-logstash .
docker build -f Dockerfile-otel     --tag sologger-otel .

docker run -d \
  -v "$(pwd)"/config/demo/log4rs-config.yml:/config/log4rs-config.yml \
  -v "$(pwd)"/config/demo/sologger-config.json:/config/sologger-config.json \
  sologger-logstash
```

No `-i`/`-t` needed: sologger never reads stdin. Released images are `brytelands/sologger-logstash`
and `brytelands/sologger-otel` (linux/amd64 + arm64); images published before v0.4.0 crash at startup
without `-t` ("Error starting WebSocket for log subscription: unexpected end of file").

The images set `SOLOGGER_APP_CONFIG_LOC=/config/sologger-config.json`, so a mounted config must land
at exactly that path — or override the env var to match where you mounted it.

## Full stacks

`docker-examples/` has complete compose stacks. Parseable is the lightest; OpenSearch is the one the
README recommends.

```bash
cd docker-examples/docker-parseable && docker compose up
```

Every stack (and the root `docker-compose.yaml`) builds sologger from the repo source as
`sologger-logstash:local` / `sologger-otel:local` with `pull_policy: build`, so `docker compose up`
picks up local code changes; the first build takes several minutes. The SigNoz swarm stack is the
exception — `docker stack deploy` ignores `build:`, so run `docker compose build` first.

To watch different programs, edit the `sologger-config.json` inside that example directory — each
stack mounts its own copy.

These bring up multi-container log backends. Confirm with the user before starting one, and tell them
how to tear it down (`docker compose down`).

## Verifying it works

Startup is silent on success. Signs of life:

- LogStash build: output appears wherever `log4rs-config.yml` sends it. `config/local/` uses a
  `console` appender at `level: info`, so structured logs print to **stdout** — you should see lines
  within seconds on devnet. Other config directories may route to a file or a LogStash socket
  instead; read the appenders before concluding nothing happened.
- OTel build: logs go to the configured OTLP endpoint; nothing local to look at unless the stdout
  exporter is enabled. To check it end to end, run `otel/opentelemetry-collector-contrib` with an
  OTLP gRPC receiver on 4317 and `file` exporters, and see `AGENTS.md` ("Running the binary") for
  what to compare. Set `OTEL_METRIC_EXPORT_INTERVAL=10000` to see metrics within a short run.
- No transport: colored, CPI-indented transaction trees on stdout.

`RUST_LOG` does nothing — sologger's log levels come from the transport configs.

Stopping: Ctrl-C or `docker stop` (SIGINT/SIGTERM) delivers buffered OpenTelemetry data first, waiting
up to 5 seconds. A second Ctrl-C exits immediately.

Common failures:

- *"log4rs config '…' not found — logstash transport disabled"* (or the otel equivalent) —
  the config path does not resolve from your working directory. Run from the repo root.
- *"invalid log4rs config"* / *"invalid OpenTelemetry config"* / *"failed to initialize the OTel log
  exporter"* — the transport config is broken (bad YAML or JSON, a missing `endpoint`/`logLevel`, a
  malformed endpoint URL); the message names the file and the problem. That transport is disabled.
- No logs ever arrive — the selected programs may simply not be active on that network, or the RPC
  dropped the subscription. Try `"programs": ["*"]` on devnet to confirm the pipeline works.

## Browser alternative

For quickly checking parser behavior without an RPC subscription, `sologger-ui`'s `/convert` page
runs the same parsing logic through WASM on pasted log text:

```bash
cd sologger-ui && npm run dev
```

Note that page runs the **committed WASM build**, which may lag the Rust source — see the `wasm-sync`
skill.
