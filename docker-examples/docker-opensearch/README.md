** sologger with opensearch example**

TODO

Possible issues running, try:

If you run into:

```shell
2023-09-30 15:23:44 ERROR: [1] bootstrap checks failed
2023-09-30 15:23:44 [1]: max virtual memory areas vm.max_map_count [65530] is too low, increase to at least [262144]
```
try:

```shell
sudo sysctl -w vm.max_map_count=262144
```