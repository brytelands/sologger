import {WasmLogContextTransformer} from '../pkg/sologger_log_transformer_wasm.js';

async function run() {
    const logs = ["Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]",
        "Program log: Instruction: Initialize",
        "Program 11111111111111111111111111111111 invoke [2]",
        "Program 11111111111111111111111111111111 success",
        "Program log: Initialized new event. Current value",
        "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units",
        "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success",
        "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]",
        "Program log: Create",
        "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units",
        "Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)",
        "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete"
    ];

    const transformer = new WasmLogContextTransformer(["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7", "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]);

    // Example usage of from_rpc_logs_response
    const rpcLogsResponse = {
        signature: "test_signature",
        err: null,
        logs: logs
    };
    const slot = 123456789;

    try {
        const result = transformer.from_rpc_logs_response(rpcLogsResponse, BigInt(slot));
        console.log(JSON.parse(JSON.stringify(result)));
    } catch (error) {
        console.error("Error:", error);
    }

    // Example usage of from_rpc_response
    const rpcResponse = {
        context: {
            slot: 12324,
            api_version: null
        },
        value: rpcLogsResponse
    };

    try {
        const result = transformer.from_rpc_response(rpcResponse);
        console.log(JSON.parse(JSON.stringify(result)));
    } catch (error) {
        console.error("Error:", error);
    }
}

run().catch(console.error);