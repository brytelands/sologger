## Configuration Examples

This directory contains examples of configuration files for logging to Logstash, OpenTelemetry, stdout, and files.
Run them from the repository root: paths inside the configs are relative to the working directory.

**demo**

This is a simple log4rs configuration file that listens for all programs logs and prints logs to stdout.

**file-example**

This is a log4rs configuration file that logs that listens to all Solana system program logs and write them to a file.

**local**

This is a simple log4rs configuration file that listens to all Solana system programs logs and prints logs to stdout.

**logstash-example**

This is a simple log4rs configuration file that listens to all Solana system programs logs and sends the logs to Logstash.

**otel-example**

This is a simple OpenTelemetry configuration file that listens to all Solana system programs logs and sends the logs to an OpenTelemetry collector.

**combined-example**

A sologger-config that enables both transports at once, reusing `logstash-example/log4rs-config.yml` and `otel-example/opentelemetry-config.json`. Needs a binary built with `--features 'enable_logstash enable_otel'`; every log goes to LogStash and to the OpenTelemetry collector, each at its own level.

**webhook-example**

A sologger-config plus webhook config that posts matching records to Discord, Slack or any HTTP endpoint. Needs `--features enable_webhook`.
