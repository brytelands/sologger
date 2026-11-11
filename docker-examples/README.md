## Log Management examples with Docker

Every stack builds sologger from this repository's source (`Dockerfile-logstash` or `Dockerfile-otel` at the repo root, tagged `sologger-logstash:local` / `sologger-otel:local`) instead of pulling an image from Docker Hub, so the stacks always run the code you have checked out. `pull_policy: build` makes every `docker compose up` rebuild it; the first build compiles sologger and takes several minutes, later ones reuse Docker's cache and take seconds. To run a released image instead, replace the service's `image`/`build`/`pull_policy` with `image: brytelands/sologger-logstash:<version>` (or `sologger-otel`).

To use a different program selection or RPC, edit the stack's own `sologger-config.json`. `docker compose stop` lets sologger deliver buffered OpenTelemetry data before it exits.

**docker-elk**

This is a simple example of how to use the ELK stack to manage Solana logs. It uses the sologger-logstash image to send logs to Logstash, which then sends the logs to Elasticsearch. Kibana is used to visualize the logs.

**docker-opensearch**

This is a simple example of how to use OpenSearch to manage Solana logs. It uses the sologger-logstash image to send logs to send logs to Logstash, which then sends the logs to OpenSearch.

**docker-parseable**

This is the most straightforward way to play around with Sologger or quickly get up and running for development or testing purposes.
This is a simple example of how to use the sologger-logstash image to send logs to Logstash, which then sends the logs to Parseable.

**docker-signoz**

This is an example of how to use Signoz to manage Solana observability. It uses the sologger-otel image to send logs — plus per-transaction CPI traces and compute-unit/failure metrics (`enableTraces`/`enableMetrics`) — to the OpenTelemetry collector provided by Signoz.