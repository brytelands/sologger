# Sologger / Parseable / Winston Example

## Overview

This is an example of using the Sologger WASM libraries in a node application. It will perform the following:

- Establish a websocket with the Solana RPC logSubscribe endpoint on devnet
- Listen for all program logs
- Structure the logs using the Sologger libraries
- Send the structured logs to Parseable


```shell
cd docker
docker compose -f docker-compose.yaml up 
```

Login into Parseable (http://localhost:8000/) with:

user name: admin

password: admin

At the home page, click the "Create Stream" button and give the new stream the name: solanadevnet

Now you can run the app to listen to logs from Solana devnet, give them structure and pipe them to Parseable.

```shell
node app.js
```

After a minute or so, your logs should start showing up in your local Parseable. You can view them here: http://localhost:8000/solanadevnet/explore