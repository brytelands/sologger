## Log Management examples with Docker

**docker-elk**

This is a simple example of how to use the ELK stack to manage Solana logs. It uses the sologger-logstash image to send logs to Logstash, which then sends the logs to Elasticsearch. Kibana is used to visualize the logs.

**docker-opensearch**

This is a simple example of how to use OpenSearch to manage Solana logs. It uses the sologger-logstash image to send logs to send logs to Logstash, which then sends the logs to OpenSearch.

**docker-parseable**

This is the most straightforward way to play around with Sologger or quickly get up and running for development or testing purposes.
This is a simple example of how to use the sologger-logstash image to send logs to Logstash, which then sends the logs to Parseable.

**docker-signoz**

This is an example of how to use Signoz to manage Solana observability. It uses the sologger-otel image to send logs — plus per-transaction CPI traces and compute-unit/failure metrics (`enableTraces`/`enableMetrics`) — to the OpenTelemetry collector provided by Signoz.