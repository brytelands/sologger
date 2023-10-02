//! # solana-log-context
//!
//!**Overview**
//!
//!This library provides functionality to turn raw logs output by Solana RPCs into structured logs for specified program IDs.
//!
//!**Usage**
//!
//!```rust
//!    //Provide the ProgramSelector with the Program IDs for which you want to parse logs.
//!    //If you want to parse logs for all programs, use ProgramsSelector::new(&["*".to_string()])
//!    let programs_selector = ProgramsSelector::new(&["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7".to_string()]);
//!    //Provide the raw logs, transfer error, programs selector, slot, and signature to the LogContext::parse_logs function.
//!    let log_contexts = LogContext::parse_logs(&logs, "".to_string(), &programs_selector, 1, "12345".to_string());
//!```
//!
//!For example, if we have a list of raw logs retrieved from the Solana RPC, we can parse them into structured logs using the LogContext::parse_logs function. The first parameter is the raw logs, the second parameter is the program ID, the third parameter is the programs selector, the fourth parameter is the slot, and the fifth parameter is the signature.
//!
//!The LogContext::parse_logs function returns a vector of LogContexts. Each LogContext contains a vector of LogMessages. Each LogMessage contains a vector of LogFields. Each LogField contains a key and a value.
//!
//!Here is an example of raw logs retrieved from the Solana RPC:
//!
//!```text
//!Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]
//!Program log: Instruction: Initialize
//!Program 11111111111111111111111111111111 invoke [2]
//!Program 11111111111111111111111111111111 success
//!Program log: Initialized new event. Current value
//!Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units
//!Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success
//!Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]
//!Program log: Create
//!Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units
//!Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)
//!Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete
//!```
//!
//!Here is an example of the structured logs that are returned from the LogContext::parse_logs function:
//!
//!```json
//!{
//!  "log_messages":[
//!    "Instruction: Initialize",
//!    "Initialized new event. Current value"
//!  ],
//!  "data_logs":[
//!
//!  ],
//!  "raw_logs":[
//!    "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]",
//!    "Program log: Instruction: Initialize",
//!    "Program log: Initialized new event. Current value",
//!    "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units",
//!    "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success"
//!  ],
//!  "errors":[
//!
//!  ],
//!  "transaction_error":"",
//!  "program_id":"9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7",
//!  "parent_program_id":"",
//!  "depth":1,
//!  "id":0,
//!  "instruction_index":0,
//!  "invoke_result":"",
//!  "slot":1,
//!  "signature":"12345"
//!}
//!```
//!
//!```json
//!{
//!  "log_messages":[
//!
//!  ],
//!  "data_logs":[
//!
//!  ],
//!  "raw_logs":[
//!    "Program 11111111111111111111111111111111 invoke [2]",
//!    "Program 11111111111111111111111111111111 success"
//!  ],
//!  "errors":[
//!
//!  ],
//!  "transaction_error":"",
//!  "program_id":"11111111111111111111111111111111",
//!  "parent_program_id":"9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7",
//!  "depth":2,
//!  "id":1,
//!  "instruction_index":0,
//!  "invoke_result":"",
//!  "slot":1,
//!  "signature":"12345"
//!}
//!```
//!
//!```json
//!{
//!  "log_messages":[
//!    "Create"
//!  ],
//!  "data_logs":[
//!
//!  ],
//!  "raw_logs":[
//!    "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]",
//!    "Program log: Create",
//!    "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units",
//!    "Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)",
//!    "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete"
//!  ],
//!  "errors":[
//!    "Invoked an instruction with data that is too large (12178014311288245306 > 10240)",
//!    "Program failed to complete"
//!  ],
//!  "transaction_error":"",
//!  "program_id":"AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
//!  "parent_program_id":"",
//!  "depth":1,
//!  "id":2,
//!  "instruction_index":1,
//!  "invoke_result":"",
//!  "slot":1,
//!  "signature":"12345"
//!}
//!```
//!
//!**Additional Usage**
//!
//!The LogContext also provides utility to retrieve specific information from a log line.
//!
//!- get_program_data: Returns the data mentioned in the provided log (for logs prefixed with "Program data: ")
//!- parse_logs_from_string: Parses the provided payload and returns a vector of LogContexts. The payload in this case is the raw JSON response as a string from the Solana RPC log_subscription endpoint.
//!- has_errors: Returns true if the log contains a program error
//!
//!**Technical Details**
//!
//!The parsing of the raw logs is done using a regular expression. The regular expression is defined in the LogContext::get_log_regex function. The regular expression is defined as follows:
//!
//![regex-vis](https://regex-vis.com/?r=%28%3F%3CprogramInvoke%3E%5EProgram+%28%3F%3CinvokeProgramId%3E%5B1-9A-HJ-NP-Za-km-z%5D%7B32%2C%7D%29+invoke+%5C%5B%28%3F%3Clevel%3E%5Cd%2B%29%5C%5D%24%29%7C%28%3F%3CprogramSuccessResult%3E%5EProgram+%28%3F%3CsuccessResultProgramId%3E%5B1-9A-HJ-NP-Za-km-z%5D%7B32%2C%7D%29+success%24%29%7C%28%3F%3CprogramFailedResult%3E%5EProgram+%28%3F%3CfailedResultProgramId%3E%5B1-9A-HJ-NP-Za-km-z%5D%7B32%2C%7D%29+failed%3A+%28%3F%3CfailedResultErr%3E.*%29%24%29%7C%28%3F%3CprogramCompleteFailedResult%3E%5EProgram+failed+to+complete%3A+%28%3F%3CfailedCompleteError%3E.*%29%24%29%7C%28%3F%3CprogramLog%3E%5E%5EProgram+log%3A+%28%3F%3ClogMessage%3E.*%29%24%29%7C%28%3F%3CprogramData%3E%5EProgram+data%3A+%28%3F%3Cdata%3E.*%29%24%29%7C%28%3F%3CprogramConsumed%3E%5EProgram+%28%3F%3CconsumedProgramId%3E%5B1-9A-HJ-NP-Za-km-z%5D%7B32%2C%7D%29+consumed+%28%3F%3CconsumedComputeUnits%3E%5Cd*%29+of+%28%3F%3CallComputedUnits%3E%5Cd*%29+compute+units%24%29%7C%28%3F%3CprogramConsumption%3E%5E%5EProgram+consumption%3A+%28%3F%3CcomputeUnitsRemaining%3E.*%29%24%29%7C%28%3F%3ClogTruncated%3E%5ELog+truncated%24%29%7C%28%3F%3CprogramReturn%3E%5EProgram+return%3A+%28%3F%3CreturnProgramId%3E%5B1-9A-HJ-NP-Za-km-z%5D%7B32%2C%7D%29+%28%3F%3CreturnMessage%3E.*%29%24%29&e=0)
//!
//!```regexp
//!(?<programInvoke>^Program (?<invokeProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) invoke \[(?<level>\d+)\]$)|(?<programSuccessResult>^Program (?<successResultProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) success$)|(?<programFailedResult>^Program (?<failedResultProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) failed: (?<failedResultErr>.*)$)|(?<programCompleteFailedResult>^Program failed to complete: (?<failedCompleteError>.*)$)|(?<programLog>^^Program log: (?<logMessage>.*)$)|(?<programData>^Program data: (?<data>.*)$)|(?<programConsumed>^Program (?<consumedProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) consumed (?<consumedComputeUnits>\d*) of (?<allComputedUnits>\d*) compute units$)|(?<programConsumption>^^Program consumption: (?<computeUnitsRemaining>.*)$)|(?<logTruncated>^Log truncated$)|(?<programReturn>^Program return: (?<returnProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) (?<returnMessage>.*)$)
//!```
//!
//!The LogContext attempts to loop through the raw logs returned from the Solana websocket log subscription frames, or groups of logs retrieved from a specific transaction or block.
//!If logs are provided that are out of order or not from a contained unit such as a block, transaction or websocket frame, then the LogContext will most likely fail.
//!
//! Here is the JSON schema for the LogContext:
//! 
//!```json
//! {
//!  "$schema": "http://json-schema.org/draft-04/schema#",
//!  "title": "LogContext",
//!  "description": "This is the structured log format for a Solana Program log message.",
//!  "type": "object",
//!  "properties": {
//!    "log_messages": {
//!      "description": "The log messages produced by the program, via the msg! or emit! macros. These logs being with 'Program log:'",
//!      "type": "array",
//!      "items": {
//!        "type": "string"
//!      }
//!    },
//!    "data_logs": {
//!      "description": "The data messages containing serialized data produced by the program, usually via the emit! or emit_cpi! macros provided by Anchor. These logs begin with 'Program data:'",
//!      "type": "array",
//!      "items": {
//!        "type": "string"
//!      }
//!    },
//!    "raw_logs": {
//!      "description": "This is the raw log output from the program. This will contain all logs, regardless of prefix.",
//!      "type": "array",
//!      "items": {
//!        "type": "string"
//!      }
//!    },
//!    "errors": {
//!      "description": "The errors produced by the program. These logs being with 'Program failed to complete:', for example 'Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)'",
//!      "type": "array",
//!      "items": {
//!        "type": "string"
//!      }
//!    },
//!    "transaction_error": {
//!      "description": "The transaction error produced by the program. This value is not parsed from the raw logs, but is provided by the RPC log subscription response as to why a transaction might be rejected.",
//!      "type": "string"
//!    },
//!    "program_id": {
//!      "description": "The program id of the program that produced the logs.",
//!      "type": "string"
//!    },
//!    "parent_program_id": {
//!      "description": "The program id of the program that invoked the program that produced the logs.",
//!      "type": "string"
//!    },
//!    "depth": {
//!      "description": "The depth of the program invocation. This is 1 for the first program invoked, 2 for the second, and so on.",
//!      "type": "integer"
//!    },
//!    "id": {
//!      "description": "The unique ID of this log context. It is the combined value of the program ID, slot and log parsing algorithm index.",
//!      "type": "integer"
//!    },
//!    "instruction_index": {
//!      "description": "This is the index of the instruction that produced the logs. This is used to determine the order of logs produced by a program.",
//!      "type": "integer"
//!    },
//!    "invoke_result": {
//!      "description": "The result of the program invocation from logs prefixed with 'Program return'. Example: 'Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA='",
//!      "type": "string"
//!    },
//!    "slot": {
//!      "description": "The period of time for which each leader ingests transactions and produces a block.",
//!      "type": "integer"
//!    },
//!    "signature": {
//!      "description": "The signature of the transaction that invoked the program.",
//!      "type": "string"
//!    }
//!  }
//!}
//!```

pub mod programs_selector;
mod rpc_response;
pub mod solana_log_context;
