# Sologger using SigNoz with OpenTelemetry

## Deploy

### Using Docker Compose

If you don't have docker-compose set up, please follow [this guide](https://docs.docker.com/compose/install/)
to set up docker compose before proceeding with the next steps.

```sh
docker-compose -f docker/clickhouse-setup/docker-compose.yaml up -d
```

Open http://localhost:3301 in your favourite browser. You'll be asked to create an account. Once in, you'll be able to view the dashboard and navigate to the logs section.
