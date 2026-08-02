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
