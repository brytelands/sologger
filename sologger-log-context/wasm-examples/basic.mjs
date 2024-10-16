import { WasmLogParser, parse_logs_basic } from '../pkg/sologger_log_context.js';

async function run() {
    const parser = new WasmLogParser(["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7", "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]);
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
    const result = parser.parse_logs(logs, "", BigInt(123456789), "signature");

    console.log(JSON.parse(JSON.stringify(result)));

    // Or use the basic function
    const basicResult = parse_logs_basic(logs, ["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7", "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]);
    console.log(JSON.parse(JSON.stringify(basicResult)));
}

run().catch(console.error);