# sologger

Configurable standalone service to parse raw logs emitted from a Solana RPC into structured logs and transport Solana
logs to either a LogStash or OpenTelemetry endpoint via TCP. This helps improve the observability of your programs
running on chain.

Logs that contain errors will have the log level set to ERROR. All other logs will have the log level set to INFO.

### Quick Start

**See the parent README for running with Docker. This is specific to the sologger binary.

```shell
#This will start listening to all Solana system programs and printing the structured logs to std out
#If running from the project root directory, then you can run the following command:
cargo run --features enable_logstash ./config/local/sologger-config.json
```

### Configure

By default, the sologger binary will look for the config file at "./config/local/sologger-config.json" when run from the
project root. You can override this by setting the SOLOGGER_APP_CONFIG_LOC environment variable to the location of your
config file or specifying it as the first argument using cargo run.

The spec for the configuration can be found in the [sologger-config-schema.json](sologger-config-schema.json) file.

Update the sologger-config.json and log4rs-config.yml or opentelemetry-config.json options in ./config directory to your
needs.

````
{
    "log4rs_config_location": "./config/logstash-config.yml",
    "rpc_url": "wss://<ADD WEBSOCKED ADDRESS HERE>,
    "program_ids": [""]
}

log4rs_config_location: This is the location of your logging configuration. This contains the configuration for your logger and Logstash transport.
rpc_url: This is the url which the Solana pubsub client will connect to for the log subscription.
program_ids: If you want to get logs for specific programs, then add the program ID as a string to this array. If the array contains an empty string, then all logs are retrieved.

````

### Ingestion robustness

**Reconnect + gap detection.** Each subscription is supervised: on disconnect or subscribe
failure it reconnects with exponential backoff (1s doubling to a 30s cap, reset once
messages flow again). After a reconnect, the first slot seen is compared with the last slot
seen before the drop; a gap is logged and counted in the `sologger.slots.missed` metric
(OTel builds with `enableMetrics`), alongside `sologger.websocket.reconnects`.

**Truncation backfill.** When a transaction's logs arrive truncated (`Log truncated`), the
full transaction is refetched over HTTP via `getTransaction` and re-parsed, so downstream
consumers see the complete CPI tree. On by default; disable with `"backfillTruncated": false`.
The HTTP endpoint is `rpcHttpUrl`, or derived from `rpcUrl` when unset (ws→http, port
8900→8899 for local validators).

**Historical backfill (post-mortem mode).** An optional `backfill` block replays past
transactions of the selected programs through the normal pipeline before (or instead of)
the live tail:

```json
{
  "rpcUrl": "wss://api.mainnet-beta.solana.com",
  "programsSelector": { "programs": ["CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"] },
  "backfill": {
    "fromSlot": 250000000,
    "untilSlot": 250100000,
    "limit": 500,
    "throttleMs": 200,
    "exitAfter": true
  }
}
```

`limit` caps signatures per program, `throttleMs` spaces out `getTransaction` calls for RPC
rate limits, and `exitAfter: true` exits when the replay finishes. Explicit programs are
required — there is no all-programs history API.

**blockSubscribe source.** Set `"source": "blockSubscribe"` to ingest whole blocks instead
of per-transaction log notifications (one WebSocket message per block, every matching
transaction inside). Note that many public RPC providers do not enable blockSubscribe;
`logsSubscribe` remains the default.

### Pretty console mode (no config needed)

When no transport is configured — the binary was built without transport features, or the
configured transport config files don't exist — sologger falls back to colored, CPI-indented
console output instead of exiting. That makes a bare run an everyday `tail -f` for
`solana-test-validator`:

```shell
cargo run    # no transport config: pretty console output
```

```
── slot 216778028 · 5j2K…9Qw ✗ FAILED
  CLMM9tUo… OpenPosition 90232/400000 CU ✗
    11111111… ✗
      ✗ Transfer: insufficient lamports 13792320, need 15616720
```

Colors are applied only when stdout is a terminal.

### Webhook transport (optional)

A binary built with `enable_webhook` POSTs matching records to Discord, Slack, or any HTTP
endpoint. Point `webhookConfigLocation` in sologger-config.json at a webhook config:

```json
{
  "url": "https://discord.com/api/webhooks/<id>/<token>",
  "format": "discord",
  "errorsOnly": true,
  "programs": ["CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"],
  "instructions": [],
  "timeoutMs": 5000
}
```

- `format`: `discord` ({"content": ...}), `slack` ({"text": ...}), or `json` (the raw structured
  log record). Defaults to `json`.
- `errorsOnly`, `programs`, `instructions`: matching rules, combined with AND; empty lists match
  everything.

See `config/webhook-example/` for a ready-made pair. Deliveries happen off the ingestion path;
failures are logged and dropped, not retried.

```shell
cargo run --features enable_webhook ./config/webhook-example/sologger-config.json
```

### IDL decoding (optional)

If you provide an Anchor IDL for a program, sologger decodes its logs as it parses them:
`Program data:` events are borsh-decoded into the `decoded_events` field of each structured log record, and
`custom program error` codes are resolved against the IDL's `errors` array into `error_name`. Both the legacy (pre-0.30)
and the 0.30+ IDL spec are supported.

Add an `idls` map to sologger-config.json, keyed by program ID, with paths relative to the working directory:

```json
{
  "rpcUrl": "wss://...",
  "idls": {
    "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C": "./idls/raydium_cp_swap.json"
  }
}
```

An IDL that is missing or fails to parse is reported at startup and skipped; log parsing continues without enrichment
for that program.

### Traces and metrics (optional, OTel builds)

A binary built with `enable_otel` can export each transaction as an OpenTelemetry trace and record metrics, in
addition to shipping logs. Enable them in the OpenTelemetry config file:

```json
{
  "logConfig": { "service.name": "sologger" },
  "endpoint": "http://localhost:4317",
  "tracesEndpoint": "http://localhost:4317",
  "metricsEndpoint": "http://localhost:4317",
  "logLevel": "INFO",
  "enableTraces": true,
  "enableMetrics": true
}
```

**Traces:** one trace per transaction, one span per program invocation, parented by CPI depth — a Jaeger/SigNoz
waterfall of the call tree, with program ID, instruction name, compute units and error details as span attributes.
The transaction signature is the `solana.signature` attribute on the root span. **Span durations are synthetic:**
Solana logs carry no timestamps, so each span's duration is its consumed compute units rendered as microseconds
(1 CU = 1µs). Durations show CU proportions, not wall time.

**Metrics:** `sologger.compute_units` (histogram per program and instruction), `sologger.transactions`,
`sologger.transactions.failed` (attributed to the deepest failing program), `sologger.logs.truncated`, and
`sologger.websocket.reconnects`.

When `tracesEndpoint`/`metricsEndpoint` (and `endpoint`) are empty, spans and metrics print to stdout, which is handy
for local development.

**Run**

SOLOGGER_APP_CONFIG_LOC=./config/sologger-config.json cargo run

```shell
#Run the logstash image with and mount your specific log4rs config and sologger config 
docker run -d -t --mount type=bind,source="$(pwd)"/config/demo/log4rs-config.yml,target=/config/log4rs-config.yml --mount type=bind,source="$(pwd)"/config/demo/sologger-config.json,target=/config/sologger-config.json sologger-logstash

#Run the logstash image with and mount a volume your specific log4rs config and sologger config. Do this if you are overriding SOLOGGER_APP_CONFIG_LOC and specifying a different sologger config file name and/or location
docker run -d -t -v "$(pwd)"/config/demo/log4rs-config.yml:/config/log4rs-config.yml -v "$(pwd)"/config/demo/sologger-config.json:/config/sologger-config.json sologger-logstash
```

**Design**

TODO

**Miscellaneous**

To run the service with Tokio runtime metrics enabled, run the following command:

```shell
RUSTFLAGS="--cfg tokio_unstable" cargo run --features 'enable_otel enable_tokio_rt_metrics'
```
