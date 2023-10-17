# Sologger with Logstash and OpenSearch

**Overview**

This example shows how to run the sologger service with Logstash and OpenSearch.
This example is set up to listen to Solana system programs and listen for all log levels. If you want to listen to specific programs, then update the program IDs in the sologger-config.json file.

**Run**

```shell
docker compose up
```

If everything starts up correctly, you can log into OpenSearch at:

http://localhost:5601/

With the credentials:
admin/admin

If the OpenSearch nodes don't start, you may see "max virtual memory areas" error in your output. If you run into that issue
you can try the following:

```shell
2023-09-30 15:23:44 ERROR: [1] bootstrap checks failed
2023-09-30 15:23:44 [1]: max virtual memory areas vm.max_map_count [65530] is too low, increase to at least [262144]
```

try:

```shell
sudo sysctl -w vm.max_map_count=262144
```