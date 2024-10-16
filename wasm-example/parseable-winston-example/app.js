const winston = require('winston');
const { ParseableTransport } = require('parseable-winston')
const WebSocket = require('ws');
const solanaLogParser = require('sologger_log_transformer_wasm')
const LogstashTransport = require("winston-logstash/lib/winston-logstash-latest");

const parseableTransport = new ParseableTransport({
    url: "http://localhost:8000/api/v1/logstream", // Ex: 'https://parsable.myserver.local/api/v1/logstream'
    username: "admin",
    password: "admin",
    logstream: "solanadevnet", // The log stream name
    http2: false,
})

// Here is an example if you choose to use Logstash instead of sending logs to Parseable directly.
// const logstashTransport = new LogstashTransport({
//     port: 50000,
//     node_name: "solana-devnet-logs",
//     host: "localhost"
// })

// Create a Winston logger
const logger = winston.createLogger({
    level: 'info',
    format: winston.format.simple(),
    transports: [
        new winston.transports.Console(),
        parseableTransport
    ]
});

const sanitizeLogMessage = (message) => {
    if (typeof message === 'object') {
        return JSON.stringify(message, (key, value) => {
            if (typeof value === 'string') {
                // Replace newlines and other problematic characters
                return value.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
            }
            return value;
        });
    }
    if (typeof value === 'string') {
        return value.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
    }
    return message;
};

// Solana RPC endpoint (replace with your preferred endpoint)
const SOLANA_RPC_WEBSOCKET = 'wss://api.devnet.solana.com';

async function main() {
    // Initialize the WASM module
    const { WasmLogContextTransformer } = await solanaLogParser;
    const transformer = new WasmLogContextTransformer(["*"]); // Listen to all programs

    const ws = new WebSocket(SOLANA_RPC_WEBSOCKET);

    ws.on('open', () => {
        console.log('Connected to Solana WebSocket');

        // Subscribe to all logs
        const subscribeMessage = {
            jsonrpc: '2.0',
            id: 100,
            method: 'logsSubscribe',
            "params": [ "all" ]
        };

        ws.send(JSON.stringify(subscribeMessage));
    });

    ws.on('message', async (data) => {
        const message = JSON.parse(data);

        if (message.method === 'logsNotification') {
            const logs = message.params.result.value.logs;
            const slot = message.params.result.context.slot;
            const signature = message.params.result.value.signature;
            const err = message.params.result.value.err === null ? null : message.params.result.value.err;

            try {
                const parsedLogs = transformer.from_rpc_logs_response({
                    signature,
                    err,
                    logs
                }, BigInt(slot));

                parsedLogs.forEach((solana_log) => {
                    const sanitizedLog = {
                        signature: sanitizeLogMessage(signature),
                        slot,
                        solana: JSON.parse(sanitizeLogMessage(solana_log))
                    };

                    if(err !== undefined && err != null) {
                        logger.info('Dev Solana logs', sanitizedLog);
                    } else {
                        logger.error('Dev Solana logs', sanitizedLog);
                    }
                });

                } catch (error) {
                logger.error('Error parsing logs', {
                    error: sanitizeLogMessage(error.message),
                    signature: sanitizeLogMessage(signature),
                    slot
                });
            }
        }
    });

    ws.on('error', (error) => {
        console.log("error: " + error);
        logger.error('WebSocket error', { error: error.message });
    });

    ws.on('close', () => {
        logger.info('WebSocket connection closed');
    });
}

main().catch(error => {
    console.log("error: " + error);
    logger.error('Unhandled error', { error: error.message });
});