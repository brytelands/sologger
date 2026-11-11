![rust workflow](https://github.com/brytelands/sologger/actions/workflows/rust.yml/badge.svg) :: [![codecov](https://codecov.io/gh/brytelands/sologger/graph/badge.svg?token=76I6GD4HU4)](https://codecov.io/gh/brytelands/sologger)

# sologger

sologger is a group of libraries and binaries that can be used to parse raw logs emitted from a Solana RPC into structured logs and transport Solana logs to either a LogStash or OpenTelemetry endpoint via TCP. This helps improve the observability of your programs running on chain.

**Try it out here:** [Sologger UI](https://brytelands.github.io/sologger/)

**Demo video using Sologger with OpenSearch**

[![Watch the demo](https://img.youtube.com/vi/eYz7gHfTzl0/hqdefault.jpg)](https://www.youtube.com/embed/eYz7gHfTzl0)

### Quick Start

If you just want to run Sologger with a log mangement system, then you can use one of the following docker compose files to get up and running quickly.

The compose files build sologger from this repository's source, so the first `docker compose up` compiles it and takes several minutes; later runs reuse Docker's build cache.

- [sologger with Parseable](./docker-examples/docker-parseable/) This is the easiest way to get up and running with Sologger. If you want to monitor specific programs, all you need to do is update the program IDs in the sologger-config.json file.
- [sologger with Opensearch](./docker-examples/docker-opensearch/) This is the preferred way to get up and running with Sologger. If you want to monitor specific programs, all you need to do is update the program IDs in the sologger-config.json file.

### Install

From v0.4.0, each release ships in three forms. Pick the transport you need, `logstash` or `otel`:

- **Prebuilt Linux binaries** (x86_64 and arm64), attached to each [GitHub Release](https://github.com/brytelands/sologger/releases) with a `SHA256SUMS` file:
  `sologger-<transport>-v<version>-<x86_64|aarch64>-unknown-linux-gnu.tar.gz`
- **crates.io:** `cargo install --locked sologger --features enable_logstash` (or `enable_otel`)
- **Docker** (linux/amd64 and linux/arm64): `docker pull brytelands/sologger-logstash` or `brytelands/sologger-otel`

### Build

**Building the source**
There are two main features that can be enabled when building the binaries. The first is the Logstash feature which will enable the Logstash transport. The second is the OpenTelemetry feature which will enable the OpenTelemetry transport. You can enable both features in one binary: with both configs present, every log goes to LogStash and to OpenTelemetry, each at its own configured level.

- Logstash: `enable_logstash`
- OpenTelemetry: `enable_otel`

```shell
#If you want to build the binaries with Logstash support, then run the following command:
cargo build --features 'enable_logstash'

#If you want to build the binaries with OpenTelemetry support, then run the following command:
cargo build --features 'enable_otel'

#If you want both, in one binary, then run the following command:
cargo build --features 'enable_logstash enable_otel'

cargo test --features 'enable_logstash logstash otel'
```

**Building the docker image**

```shell
#If you want to build the image with Logstash support, then run the following command:
docker build -f 'Dockerfile-logstash' --tag sologger-logstash .

#If you want to build the image with OpenTelemetry support, then run the following command:
docker build --file 'Dockerfile-otel' --tag sologger-otel .
```

### Configure

There are two configuration files that you will need to configure to get up and running with Sologger. 
The first is the sologger-config file. This file is used to configure the sologger binary.
The second is the transport config: a log4rs-config file for the LogStash transport, and/or an opentelemetry-config file for the OpenTelemetry transport. A binary built with both features uses both when both are set (`log4rsConfigLocation` and `opentelemetryConfigLocation`); see `config/combined-example/`. A transport whose config is missing or invalid is disabled with a notice, and with no transport left sologger prints pretty console output instead.

By default, sologger will look for a config file named `sologger-config.json` in ./config/local/ directory. You can override this by setting the `SOLOGGER_APP_CONFIG_LOC` environment variable to the path of your config file. For example:

```shell
SOLOGGER_APP_CONFIG_LOC=./config/sologger-config.json cargo run --features 'enable_logstash'
```

### Run

Running the sologger binary from the project root directory.

```shell
cargo run --features enable_logstash ./config/local/sologger-config.json
```

Running the sologger image with docker.

```shell
#Run the logstash image and mount your specific log4rs config and sologger config
docker run -d --mount type=bind,source="$(pwd)"/config/demo/log4rs-config.yml,target=/config/log4rs-config.yml --mount type=bind,source="$(pwd)"/config/demo/sologger-config.json,target=/config/sologger-config.json brytelands/sologger-logstash

#The same with volumes. The image reads /config/sologger-config.json; override SOLOGGER_APP_CONFIG_LOC to use a different path
docker run -d -v "$(pwd)"/config/demo/log4rs-config.yml:/config/log4rs-config.yml -v "$(pwd)"/config/demo/sologger-config.json:/config/sologger-config.json brytelands/sologger-logstash
```

sologger runs until it receives SIGINT or SIGTERM and does not read stdin, so it needs no `-i` or `-t` and runs fine under Docker, systemd or nohup. On `docker stop` (SIGTERM) it first delivers the OpenTelemetry logs, spans and metrics it is still buffering, waiting at most 5 seconds, within Docker's default 10-second grace period.

### Libraries

If you don't want to use Sologger and want help parsing logs in your application, then you can use the following libraries:

**sologger-log-context**

This library provides functionality to turn raw logs output by Solana RPCs into structured logs for specified program IDs.

**sologger**-log-transformer

This library provides utility to extract logs from various Solana API structs, such as blocks, transactions and responses.

**sologger-log-transport**

This is a library that provides support for both LogStash and OpenTelemetry exports for logs.



