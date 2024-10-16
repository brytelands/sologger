const winston = require('winston');
const ParseableTransport = require('parseable-winston');
const WebSocket = require('ws');
const { promisify } = require('util');

// Import the WASM module
const solanaLogParserWasm = import('./pkg/solana_log_parser_wasm.js');

// Configure the Parseable transport
const parseableTransport = new ParseableTransport({
    host: 'http://localhost:8000', // Replace with your Parseable server URL
    stream: 'solana-logs',
    apikey: 'your-api-key', // Replace with your actual API key if required
});

// Create a Winston logger
const logger = winston.createLogger({
    level: 'info',
    format: winston.format.json(),
    transports: [
        new winston.transports.Console(),
        parseableTransport
    ]
});

// Solana RPC endpoint (replace with your preferred endpoint)
const SOLANA_RPC_WEBSOCKET = 'wss://api.mainnet-beta.solana.com';

async function main() {
    // Initialize the WASM module
    const { WasmLogContextTransformer } = await solanaLogParserWasm;
    const transformer = new WasmLogContextTransformer(["*"]); // Listen to all programs

    const ws = new WebSocket(SOLANA_RPC_WEBSOCKET);

    ws.on('open', () => {
        console.log('Connected to Solana WebSocket');

        // Subscribe to all logs
        const subscribeMessage = {
            jsonrpc: '2.0',
            id: 1,
            method: 'logsSubscribe',
            params: [
                {
                    commitment: 'confirmed'
                },
                {
                    encoding: 'jsonParsed',
                    commitment: 'confirmed'
                }
            ]
        };

        ws.send(JSON.stringify(subscribeMessage));
    });

    ws.on('message', async (data) => {
        const message = JSON.parse(data);

        if (message.method === 'logsNotification') {
            const logs = message.params.result.value.logs;
            const slot = message.params.result.context.slot;
            const signature = message.params.result.value.signature;

            try {
                const parsedLogs = transformer.from_rpc_logs_response({
                    signature,
                    err: null,
                    logs
                }, BigInt(slot));

                // Log the parsed results
                logger.info('Parsed Solana logs', {
                    signature,
                    slot,
                    parsedLogs: JSON.parse(JSON.stringify(parsedLogs))
                });
            } catch (error) {
                logger.error('Error parsing logs', { error: error.message, signature, slot });
            }
        }
    });

    ws.on('error', (error) => {
        logger.error('WebSocket error', { error: error.message });
    });

    ws.on('close', () => {
        logger.info('WebSocket connection closed');
    });
}

main().catch(error => {
    logger.error('Unhandled error', { error: error.message });
});