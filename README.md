![rust workflow](https://github.com/brytelands/sologger/actions/workflows/rust.yml/badge.svg) :: [![codecov](https://codecov.io/gh/brytelands/sologger/graph/badge.svg?token=76I6GD4HU4)](https://codecov.io/gh/brytelands/sologger)

# sologger

sologger is a group of libraries and binaries that can be used to parse raw logs emitted from a Solana RPC into structured logs and transport Solana logs to either a LogStash or OpenTelemetry endpoint via TCP. This helps improve the observability of your programs running on chain.

**Demo video using Sologger with OpenSearch**

![Demo video](https://youtu.be/eYz7gHfTzl0)

### Quick Start

If you just want to run Sologger with a log mangement system, then you can use one of the following docker compose files to get up and running quickly.

- [sologger with Parseable](./docker-examples/docker-parseable/) This is the easiest way to get up and running with Sologger. If you want to monitor specific programs, all you need to do is update the program IDs in the sologger-config.json file.
- [sologger with Opensearch](./docker-examples/docker-opensearch/) This is the preferred way to get up and running with Sologger. If you want to monitor specific programs, all you need to do is update the program IDs in the sologger-config.json file.

### Build

**Building the source**
There are two main features that can be enabled when building the binaries. The first is the Logstash feature which will enable the Logstash transport. The second is the OpenTelemetry feature which will enable the OpenTelemetry transport. Technically you can build with both features enabled, but this is not recommended. If you need both LogStash and OTel support, the recommended approach is to build two binaries with each feature enabled, and run each separately.

- Logstash: `enable_logstash`
- OpenTelemetry: `enable_otel`

```shell
#If you want to build the binaries with Logstash support, then run the following command:
cargo build --features 'enable_logstash'

#If you want to build the binaries with OpenTelemetry support, then run the following command:
cargo build --features 'enable_otel'
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
The second is the log4rs-config file. This file is used to configure the log4rs logger OR the opentelemetry-config file. This file is used to configure the logstash binary.

By default, sologger will look for a config file named `sologger-config.json` in ./config/local/ directory. You can override this by setting the `SOLOGGER_APP_CONFIG_LOCATION` environment variable to the path of your config file. For example:

```shell
SOLOGGER_APP_CONFIG_LOCATION=./config/sologger-config.json cargo run --features 'enable_logstash'
```

### Run

Running the sologger binary from the project root directory.

```shell
cargo run --features enable_logstash ./config/local/sologger-config.json
```

Running the sologger image with docker.

```shell
#Run the logstash image with and mount your specific log4rs config and sologger config 
docker run -d -t --mount type=bind,source="$(pwd)"/config/demo/log4rs-config.yml,target=/config/log4rs-config.yml --mount type=bind,source="$(pwd)"/config/demo/sologger-config.json,target=/config/sologger-config.json sologger-logstash

#Run the logstash image with and mount a volume your specific log4rs config and sologger config. Do this if you are overriding SOLOGGER_APP_CONFIG_LOCATION and specifying a different sologger config file name and/or location
docker run -d -t -v "$(pwd)"/config/demo/log4rs-config.yml:/config/log4rs-config.yml -v "$(pwd)"/config/demo/sologger-config.json:/config/sologger-config.json sologger-logstash
```

### Libraries

If you don't want to use Sologger and want help parsing logs in your application, then you can use the following libraries:

**sologger-log-context**

This library provides functionality to turn raw logs output by Solana RPCs into structured logs for specified program IDs.

**sologger**-log-transformer

This library provides utility to extract logs from various Solana API structs, such as blocks, transactions and responses.

**sologger-log-transport**

This is a library that provides support for both LogStash and OpenTelemetry exports for logs.



