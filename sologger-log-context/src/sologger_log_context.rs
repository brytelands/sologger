use crate::programs_selector::ProgramsSelector;
use crate::rpc_response::RpcResponse;
use lazy_static::lazy_static;
use log::{debug, trace, warn};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

//TODO add Transfer and Allocate?
const LOG_REGEX: &str = r"(?<programInvoke>^Program (?<invokeProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) invoke \[(?<level>\d+)\]$)|(?<programSuccessResult>^Program (?<successResultProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) success$)|(?<programFailedResult>^Program (?<failedResultProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) failed: (?<failedResultErr>.*)$)|(?<programCompleteFailedResult>^Program failed to complete: (?<failedCompleteError>.*)$)|(?<programLog>^^Program log: (?<logMessage>.*)$)|(?<programData>^Program data: (?<data>.*)$)|(?<programConsumed>^Program (?<consumedProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) consumed (?<consumedComputeUnits>\d*) of (?<allComputedUnits>\d*) compute units$)|(?<programConsumption>^^Program consumption: (?<computeUnitsRemaining>.*)$)|(?<logTruncated>^Log truncated$)|(?<programReturn>^Program return: (?<returnProgramId>[1-9A-HJ-NP-Za-km-z]{32,}) (?<returnMessage>.*)$)";

lazy_static! {
    static ref LOG_CONTEXT_PARSER: Regex = Regex::new(LOG_REGEX).unwrap();
}

/// A LogContext is a structured log format that represents the logs of a single program invocation, per processed slot, transaction or block.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct LogContext {
    ///The log messages produced by the program, via the msg! or emit! macros. These logs being with 'Program log:'
    pub log_messages: Vec<String>,
    ///The data messages containing serialized data produced by the program, usually via the emit! or emit_cpi! macros provided by Anchor. These logs begin with 'Program data:'
    pub data_logs: Vec<String>,
    ///The raw logs produced by the program, including all logs that do not match the other log types. These logs are not parsed and are provided as-is.
    pub raw_logs: Vec<String>,
    ///The errors produced by the program. These logs being with 'Program failed to complete:', for example 'Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)'
    pub errors: Vec<String>,
    ///The transaction error produced by the program. This value is not parsed from the raw logs, but is provided by the RPC log subscription response as to why a transaction might be rejected.
    pub transaction_error: String,
    ///The program ID of the program that produced the logs
    pub program_id: String,
    ///The program ID of the parent program that invoked the program that produced the logs
    pub parent_program_id: String,
    ///The depth of the program invocation. This value is 1 for the first program invoked, 2 for the second program invoked, etc.
    pub depth: usize,
    ///The unique ID of the program invocation. This value is a combination of the program ID, the slot and the instruction index.
    pub id: String,
    ///The index of the instruction that invoked the program that produced the logs
    pub instruction_index: usize,
    ///The result of the program invocation from logs prefixed with 'Program return'
    pub invoke_result: String,
    ///The slot of the program invocation
    pub slot: usize,
    ///The signature of the transaction that invoked the program that produced the logs
    pub signature: String,
}

impl LogContext {
    /// Creates a new, empty LogContext
    pub fn new(
        program_id: String,
        depth: usize,
        id: String,
        instruction_index: usize,
        slot: usize,
        signature: String,
    ) -> Self {
        Self {
            log_messages: vec![],
            data_logs: vec![],
            raw_logs: vec![],
            errors: vec![],
            transaction_error: "".to_string(),
            program_id,
            parent_program_id: "".to_string(),
            depth,
            id,
            instruction_index,
            invoke_result: "".to_string(),
            slot,
            signature,
        }
    }

    /// Returns true if the log contains a program error
    pub fn has_errors(&self) -> bool {
        !self.transaction_error.is_empty() || !self.errors.is_empty()
    }

    /// Convenience method to convert the LogContext to a JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Returns the program ID mentioned in the provided log
    pub fn get_invoke_program_id(log: &str) -> String {
        let capture = LOG_CONTEXT_PARSER.captures(log);
        let invoke_program_id = match capture {
            None => "".to_string(),
            Some(capture) => {
                let invoke_program_id = capture.name("invokeProgramId");
                if invoke_program_id.is_some() {
                    invoke_program_id.unwrap().as_str().to_string()
                } else {
                    "".to_string()
                }
            }
        };
        invoke_program_id
    }

    /// Returns the data mentioned in the provided log (for logs prefixed with "Program data: ")
    pub fn get_program_data(log: &str) -> String {
        let capture = LOG_CONTEXT_PARSER.captures(log);
        let program_data = match capture {
            None => "".to_string(),
            Some(capture) => match capture.name("data") {
                None => "".to_string(),
                Some(x) => x.as_str().to_string(),
            },
        };
        program_data
    }

    /// Returns the program ID mentioned in the provided log, if the log is a result log (success or failure or return)
    pub fn get_end_program_id(log: &str) -> String {
        let capture = LOG_CONTEXT_PARSER.captures(log);
        match capture {
            None => "".to_string(),
            Some(capture) => {
                match capture.name("successResultProgramId") {
                    None => "".to_string(),
                    Some(x) => {
                        return x.as_str().to_string();
                    }
                };
                match capture.name("failedResultProgramId") {
                    None => "".to_string(),
                    Some(x) => {
                        return x.as_str().to_string();
                    }
                };
                match capture.name("returnProgramId") {
                    None => "".to_string(),
                    Some(x) => {
                        return x.as_str().to_string();
                    }
                }
            }
        }
    }

    /// Parses the provided payload and returns a vector of LogContexts. The payload in this case is the raw JSON response as a string from the Solana RPC log_subscription endpoint.
    pub fn parse_logs_from_string(
        payload: &str,
        programs_selector: &ProgramsSelector,
    ) -> Vec<LogContext> {
        let response: RpcResponse = serde_json::from_str(payload).unwrap();

        Self::parse_logs(
            &response.params.result.value.logs,
            response
                .params
                .result
                .value
                .err
                .unwrap_or(serde_json::Value::Null)
                .to_string(),
            programs_selector,
            response.params.result.context.slot,
            response.params.result.value.signature,
        )
    }

    /// Parses the provided payload and returns a vector of LogContexts. The payload in this case is the raw JSON response as bytes from the Solana RPC log_subscription endpoint.
    pub fn parse_logs_from_raw_data(
        payload: &Vec<u8>,
        programs_selector: &ProgramsSelector,
    ) -> Vec<LogContext> {
        let response: RpcResponse = serde_json::from_slice(payload).unwrap();

        Self::parse_logs(
            &response.params.result.value.logs,
            response
                .params
                .result
                .value
                .err
                .unwrap_or(serde_json::Value::Null)
                .to_string(),
            programs_selector,
            response.params.result.context.slot,
            response.params.result.value.signature,
        )
    }

    /// Parses the provided logs and returns a vector of LogContexts.
    pub fn parse_logs_basic(
        logs: &Vec<String>,
        programs_selector: &ProgramsSelector,
    ) -> Vec<LogContext> {
        Self::parse_logs(logs, "".to_string(), programs_selector, 0, "".to_string())
    }

    /// Parses the provided logs and returns a vector of LogContexts. It may be provided with additional information from the Solana RPC response.
    pub fn parse_logs(
        logs: &Vec<String>,
        transaction_error: String,
        programs_selector: &ProgramsSelector,
        slot: u64,
        signature: String,
    ) -> Vec<LogContext> {
        if logs.is_empty() {
            trace!("Logs are empty, returning empty vec");
            return vec![];
        }

        let mut result: Vec<LogContext> = Vec::new();
        let mut start = true;
        let mut id = 0;
        let mut current_instruction = 0;
        let mut current_depth = 0;
        let mut call_stack: Vec<String> = Vec::new();
        let mut call_ids: Vec<usize> = Vec::new();
        let mut begin_program_id: String = "".to_string();
        let mut end_parsing = false;
        for log in logs {
            let mut log_trimmed = "".to_string();
            if !LOG_CONTEXT_PARSER.is_match(log) {
                trace!(
                    "Attempting to remove newlines and trim whitespace. No match found for: {} ",
                    &log
                );
                for line in log.lines() {
                    log_trimmed.push_str(line);
                }
                log_trimmed = trim_whitespace(log_trimmed.as_str());
                trace!("Trimmed log: {}", log_trimmed);
            }

            if !programs_selector.select_all_programs {
                let program_id = Self::get_invoke_program_id(log);
                if !program_id.is_empty()
                    && programs_selector.is_program_selected_string(&program_id)
                {
                    begin_program_id = program_id.clone();
                }

                if begin_program_id.is_empty() {
                    continue;
                }

                let end_program_id = Self::get_end_program_id(log);
                end_parsing = begin_program_id.is_empty() || begin_program_id == end_program_id;
            }

            let capture: Option<Captures> = if !log_trimmed.is_empty() {
                LOG_CONTEXT_PARSER.captures(log_trimmed.as_str())
            } else {
                LOG_CONTEXT_PARSER.captures(log)
            };

            trace!(
                "parse_log programId:{} slot:{} log:{}",
                begin_program_id,
                slot,
                log
            );

            match capture {
                None => {
                    debug!("Log not matched, adding to raw_logs: {}", &log_trimmed);
                    result[call_ids[call_ids.len() - 1]]
                        .raw_logs
                        .push(log_trimmed)
                }
                Some(capture) => {
                    let program_invoke = capture.name("programInvoke");
                    let invoke_program_id = capture.name("invokeProgramId");
                    let log_truncated = capture.name("logTruncated");
                    let level = capture.name("level");
                    let success_result_program_id = capture.name("successResultProgramId");
                    let program_successful_result = capture.name("programSuccessResult");
                    let failed_result_program_id = capture.name("failedResultProgramId");
                    let program_failed_result = capture.name("programFailedResult");
                    let failed_result_err = capture.name("failedResultErr");
                    let program_complete_failed_result =
                        capture.name("programCompleteFailedResult");
                    let failed_complete_error = capture.name("failedCompleteError");
                    let log_message = capture.name("logMessage");
                    let program_log = capture.name("programLog");
                    let data = capture.name("data");
                    let program_data = capture.name("programData");
                    let program_consumed = capture.name("programConsumed");
                    let program_consumption = capture.name("programConsumption");
                    let program_return = capture.name("programReturn");
                    let return_message = capture.name("returnMessage");
                    let return_program_id = capture.name("returnProgramId");

                    match log_truncated {
                        Some(_x) => {
                            if !call_ids.is_empty() {
                                let context_log = &mut result[call_ids[call_ids.len() - 1]];
                                context_log.invoke_result = String::from("Log truncated");
                            } else {
                                let context_log = &mut result.last_mut().unwrap();
                                context_log.invoke_result = String::from("Log truncated");
                            }
                            break;
                        }
                        None => {}
                    }

                    match program_invoke {
                        Some(x) => {
                            call_stack.push(invoke_program_id.unwrap().as_str().to_string());
                            if !start {
                                id += 1;
                            };
                            start = false;
                            current_depth += 1;
                            call_ids.push(id);
                            if level.is_some_and(|x| {
                                *x.as_str().to_string() != current_depth.to_string()
                            }) {
                                trace!("Invoke depth mismatch. This is most likely caused by finding a selected program ID log nested in a program that is not monitored, log:{}, expected: {}", x.as_str(), current_depth.to_string());
                                break;
                            }
                            let program_id = call_stack[call_stack.len() - 1].clone();
                            let unique_id: String =
                                [program_id.as_str(), &slot.to_string(), &id.to_string()].join("-");
                            let mut log_context = LogContext::new(
                                program_id,
                                call_stack.len(),
                                unique_id,
                                current_instruction,
                                slot as usize,
                                signature.to_string(),
                            );
                            if call_stack.len() > 1 {
                                log_context.parent_program_id = call_stack[0].clone();
                            }
                            result.push(log_context);
                            let result_len = result.len() - 1;
                            result[result_len].raw_logs.push(log.clone());
                        }
                        None => {}
                    }

                    match program_successful_result {
                        Some(_x) => {
                            let last_program = call_stack.pop();
                            let last_call_index = call_ids.pop();
                            if last_call_index.is_none() {
                                warn!("callIds malformed");
                            }
                            if last_program
                                .is_some_and(|x| x != success_result_program_id.unwrap().as_str())
                            {
                                warn!("[ProgramSuccess] callstack mismatch");
                            }
                            result[last_call_index.unwrap()].raw_logs.push(log.clone());
                            current_depth -= 1;
                            if current_depth == 0 {
                                current_instruction += 1;
                            }
                        }
                        None => {}
                    }

                    match program_failed_result {
                        Some(_x) => {
                            let last_program = call_stack.pop().unwrap();
                            if failed_result_program_id.is_some_and(|x| x.as_str() != last_program)
                            {
                                warn!("[ProgramFailed] callstack mismatch")
                            };
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                            result[call_ids[call_ids.len() - 1]]
                                .errors
                                .push(failed_result_err.unwrap().as_str().to_string());
                            result[call_ids[call_ids.len() - 1]].transaction_error =
                                transaction_error.to_string();
                            //TODO double check this pop
                            call_ids.pop();
                        }
                        None => {}
                    }

                    match program_complete_failed_result {
                        Some(_x) => {
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                            result[call_ids[call_ids.len() - 1]]
                                .errors
                                .push(failed_complete_error.unwrap().as_str().to_string());
                            result[call_ids[call_ids.len() - 1]].transaction_error =
                                transaction_error.to_string();
                        }
                        None => {}
                    }

                    match program_log {
                        Some(_x) => {
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                            result[call_ids[call_ids.len() - 1]]
                                .log_messages
                                .push(log_message.unwrap().as_str().to_string());
                        }
                        None => {}
                    }

                    match program_data {
                        Some(_x) => {
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                            result[call_ids[call_ids.len() - 1]]
                                .data_logs
                                .push(data.unwrap().as_str().to_string());
                        }
                        None => {}
                    }

                    match program_consumed {
                        Some(_x) => {
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                        }
                        None => {}
                    }

                    match program_consumption {
                        Some(_x) => {
                            result[call_ids[call_ids.len() - 1]]
                                .raw_logs
                                .push(log.clone());
                        }
                        None => {}
                    }

                    match program_return {
                        Some(_x) => {
                            if return_program_id
                                .is_some_and(|x| x.as_str() != call_stack[call_stack.len() - 1])
                            {
                                warn!("[InvokeReturn]: callstack mismatch")
                            }
                            result[call_ids[call_ids.len() - 1]].invoke_result =
                                return_message.unwrap().as_str().to_string();
                        }
                        None => {}
                    }
                }
            }
            if end_parsing {
                trace!("parse_logs for slot: {}", slot);
                begin_program_id = String::new();
                continue;
            }
        }

        result
    }
}

// This method is used to trim whitespace from a string, removing any duplicate whitespace characters.
// It is used to clean up the logs before parsing them.
fn trim_whitespace(s: &str) -> String {
    let mut trimmed = s.trim().to_owned();
    let mut prev_c = ' ';
    trimmed.retain(|c| {
        let result = c != ' ' || prev_c != ' ';
        prev_c = c;
        result
    });
    trimmed
}

#[cfg(test)]
mod tests {
    use crate::programs_selector::ProgramsSelector;
    use crate::sologger_log_context::LogContext;
    use std::time::SystemTime;

    //TODO fix test for ID
    #[test]
    fn log_parser_program_success_all_programs_test() {
        let mut logs: Vec<String> = vec![];
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string());
        logs.push("Program log: Instruction: Initialize".to_string());
        logs.push("Program 11111111111111111111111111111111 invoke [2]".to_string());
        logs.push("Program 11111111111111111111111111111111 success".to_string());
        logs.push("Program log: Initialized new event. Current value".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string());
        logs.push("Program log: Create".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units".to_string());
        logs.push("Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete".to_string());

        let programs_selector = ProgramsSelector::new(&["*".to_string()]);

        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_nanos_start = duration_since_epoch.as_nanos(); // u128
        let log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &programs_selector,
            1,
            "12345".to_string(),
        );

        let duration_since_epoch_end = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_nanos_end = duration_since_epoch_end.as_nanos(); // u128

        let context_len = log_contexts.len();
        assert_eq!(context_len, 3);

        assert_eq!(
            log_contexts[0].program_id,
            "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7"
        );
        assert_eq!(log_contexts[0].log_messages.len(), 2);
        assert_eq!(log_contexts[0].errors.len(), 0);
        assert_eq!(log_contexts[0].transaction_error, "".to_string());
        assert_eq!(log_contexts[0].data_logs.len(), 0);
        assert_eq!(log_contexts[0].raw_logs.len(), 5);
        assert_eq!(log_contexts[0].slot, 1);
        assert_eq!(log_contexts[0].signature, "12345");
        assert_eq!(log_contexts[0].parent_program_id, "");
        assert_eq!(log_contexts[0].depth, 1);
        // assert_eq!(log_contexts[0].id, 0);
        assert_eq!(log_contexts[0].invoke_result, "");
        assert_eq!(log_contexts[0].instruction_index, 0);

        assert_eq!(
            log_contexts[1].program_id,
            "11111111111111111111111111111111"
        );
        assert_eq!(log_contexts[1].log_messages.len(), 0);
        assert_eq!(log_contexts[1].errors.len(), 0);
        assert_eq!(log_contexts[1].transaction_error, "".to_string());
        assert_eq!(log_contexts[1].data_logs.len(), 0);
        assert_eq!(log_contexts[1].raw_logs.len(), 2);
        assert_eq!(log_contexts[1].slot, 1);
        assert_eq!(log_contexts[1].signature, "12345");
        assert_eq!(
            log_contexts[1].parent_program_id,
            "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7"
        );
        assert_eq!(log_contexts[1].depth, 2);
        // assert_eq!(log_contexts[1].id, 1);
        assert_eq!(log_contexts[1].invoke_result, "");
        assert_eq!(log_contexts[1].instruction_index, 0);

        assert_eq!(
            log_contexts[2].program_id,
            "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        );
        assert_eq!(log_contexts[2].log_messages.len(), 1);
        assert_eq!(log_contexts[2].errors.len(), 2);
        assert_eq!(log_contexts[2].transaction_error, "".to_string());
        assert_eq!(log_contexts[2].data_logs.len(), 0);
        assert_eq!(log_contexts[2].raw_logs.len(), 5);
        assert_eq!(log_contexts[2].slot, 1);
        assert_eq!(log_contexts[2].signature, "12345");
        assert_eq!(log_contexts[2].parent_program_id, "");
        assert_eq!(log_contexts[2].depth, 1);
        // assert_eq!(log_contexts[2].id, 2);
        assert_eq!(log_contexts[2].invoke_result, "");
        assert_eq!(log_contexts[2].instruction_index, 1);
    }

    #[test]
    fn log_parser_program_success_single_program_test() {
        let mut logs: Vec<String> = vec![];
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string());
        logs.push("Program log: Instruction: Initialize".to_string());
        logs.push("Program 11111111111111111111111111111111 invoke [2]".to_string());
        logs.push("Program 11111111111111111111111111111111 success".to_string());
        logs.push("Program log: Initialized new event. Current value".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string());
        logs.push("Program log: Create".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units".to_string());
        logs.push("Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete".to_string());

        let programs_selector =
            ProgramsSelector::new(&["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7".to_string()]);

        let log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &programs_selector,
            1,
            "12345".to_string(),
        );

        let context_len = log_contexts.len();
        assert_eq!(context_len, 2);

        assert_eq!(
            log_contexts[0].program_id,
            "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7"
        );
        assert_eq!(log_contexts[0].log_messages.len(), 2);
        assert_eq!(log_contexts[0].errors.len(), 0);
        assert_eq!(log_contexts[0].transaction_error, "".to_string());
        assert_eq!(log_contexts[0].data_logs.len(), 0);
        assert_eq!(log_contexts[0].raw_logs.len(), 5);
        assert_eq!(log_contexts[0].slot, 1);
        assert_eq!(log_contexts[0].signature, "12345");
        assert_eq!(log_contexts[0].parent_program_id, "");
        assert_eq!(log_contexts[0].depth, 1);
        // assert_eq!(log_contexts[0].id, 0);
        assert_eq!(log_contexts[0].invoke_result, "");
        assert_eq!(log_contexts[0].instruction_index, 0);

        assert_eq!(
            log_contexts[1].program_id,
            "11111111111111111111111111111111"
        );
        assert_eq!(log_contexts[1].log_messages.len(), 0);
        assert_eq!(log_contexts[1].errors.len(), 0);
        assert_eq!(log_contexts[1].transaction_error, "".to_string());
        assert_eq!(log_contexts[1].data_logs.len(), 0);
        assert_eq!(log_contexts[1].raw_logs.len(), 2);
        assert_eq!(log_contexts[1].slot, 1);
        assert_eq!(log_contexts[1].signature, "12345");
        assert_eq!(
            log_contexts[1].parent_program_id,
            "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7"
        );
        assert_eq!(log_contexts[1].depth, 2);
        // assert_eq!(log_contexts[1].id, 1);
        assert_eq!(log_contexts[1].invoke_result, "");
        assert_eq!(log_contexts[1].instruction_index, 0);
    }

    #[test]
    fn log_parser_program_success_single_program_filter_last_test() {
        let mut logs: Vec<String> = vec![];
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string());
        logs.push("Program log: Instruction: Initialize".to_string());
        logs.push("Program 11111111111111111111111111111111 invoke [2]".to_string());
        logs.push("Program 11111111111111111111111111111111 success".to_string());
        logs.push("Program log: Initialized new event. Current value".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units".to_string());
        logs.push("Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string());
        logs.push("Program log: Create".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units".to_string());
        logs.push("Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete".to_string());

        let programs_selector =
            ProgramsSelector::new(&["AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string()]);

        let log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &programs_selector,
            1,
            "12345".to_string(),
        );

        let context_len = log_contexts.len();
        assert_eq!(context_len, 1);

        assert_eq!(
            log_contexts[0].program_id,
            "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        );
        assert_eq!(log_contexts[0].log_messages.len(), 1);
        assert_eq!(log_contexts[0].errors.len(), 2);
        assert_eq!(log_contexts[0].transaction_error, "".to_string());
        assert_eq!(log_contexts[0].data_logs.len(), 0);
        assert_eq!(log_contexts[0].raw_logs.len(), 5);
        assert_eq!(log_contexts[0].slot, 1);
        assert_eq!(log_contexts[0].signature, "12345");
        assert_eq!(log_contexts[0].parent_program_id, "");
        assert_eq!(log_contexts[0].depth, 1);
        // assert_eq!(log_contexts[0].id, 0);
        assert_eq!(log_contexts[0].invoke_result, "");
        assert_eq!(log_contexts[0].instruction_index, 0);
    }

    #[test]
    fn log_parser_truncate_test() {
        let raw_logs: Vec<String> = vec!["Program ComputeBudget111111111111111111111111111111 invoke [1]",
                                         "Program ComputeBudget111111111111111111111111111111 success",
                                         "Program ComputeBudget111111111111111111111111111111 invoke [1]",
                                         "Program ComputeBudget111111111111111111111111111111 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.97775733506 cost_lamports_unrounded: 92752.09726081353 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1345863 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 56082 of 1393000 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1299600 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1336918 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1253337 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1290655 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1207074 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1244392 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1160811 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1198129 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1114548 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1151866 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1068285 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1105603 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 1022022 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1059340 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 975759 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 1013077 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 929496 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 966814 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 883233 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 920551 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 836970 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 874288 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 790707 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 828025 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 invoke [1]",
                                         "Program log: Instruction: Mint",
                                         "Program log: burn_per_slot: 420163.59376257134 cost_lamports_unrounded: 92752.09291779375 cost_lamports: 92000",
                                         "Program 11111111111111111111111111111111 invoke [2]",
                                         "Program 11111111111111111111111111111111 success",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                                         "Program log: Instruction: MintTo",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4401 of 744444 compute units",
                                         "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 consumed 46263 of 781762 compute units",
                                         "Program s1owa2k7P2kkLEenZPKuGddWMVpy8Pt2oMVeBdtSHM6 success",
                                         "Log truncated",
                                         "Program log: Instruction: Mint"].into_iter().map(|s| s.to_string()).collect();

        let programs_selector =
            ProgramsSelector::new_all_programs();

        let log_contexts = LogContext::parse_logs(
            &raw_logs,
            "".to_string(),
            &programs_selector,
            2523,
            "32432432".to_string(),
        );

        let context_len = log_contexts.len();

        assert_eq!(context_len, 44);
        assert_eq!(log_contexts.last().unwrap().invoke_result, "Log truncated");
    }

    #[test]
    fn log_parser_program_failure_test() {
        let mut logs: Vec<String> = vec![];
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string());
        logs.push("Program log: Create".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units".to_string());
        logs.push("Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)".to_string());
        logs.push("Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete".to_string());
        let programs_selector =
            ProgramsSelector::new(&["AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string()]);

        let log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &programs_selector,
            2523,
            "32432432".to_string(),
        );

        let context_len = log_contexts.len();
        assert_eq!(context_len, 1);
    }

    #[test]
    fn invoke_program_id_test() {
        let log = "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string();

        let invoke_program_id = LogContext::get_invoke_program_id(&log);
        assert_eq!(
            invoke_program_id,
            "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7"
        );
    }

    #[test]
    fn log_parser_get_data_test() {
        let raw_logs: Vec<String> = vec![
        "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD invoke [1]",
        "Program log: Instruction: PlacePerpOrderV3",
        "Program log: GetOraclePrice Pyth price: price=18211908 age=0",
        "Program data: f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA",
        "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD consumed 36432 of 1015220 compute units",
        "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD success"
        ].into_iter().map(|s| s.to_string()).collect();

        let programs_selector = ProgramsSelector::new_all_programs();

        let log_contexts = LogContext::parse_logs(
            &raw_logs,
            "".to_string(),
            &programs_selector,
            2523,
            "32432432".to_string(),
        );

        assert_eq!(
            log_contexts[0].program_id,
            "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD"
        );
        assert_eq!(log_contexts[0].log_messages.len(), 2);
        assert_eq!(log_contexts[0].errors.len(), 0);
        assert_eq!(log_contexts[0].transaction_error, "".to_string());
        assert_eq!(log_contexts[0].data_logs.len(), 1);
        assert_eq!(log_contexts[0].raw_logs.len(), 6);
        assert_eq!(log_contexts[0].slot, 2523);
        assert_eq!(log_contexts[0].signature, "32432432");
        assert_eq!(log_contexts[0].parent_program_id, "");
        assert_eq!(log_contexts[0].depth, 1);
        // assert_eq!(log_contexts[0].id, 0);
        assert_eq!(log_contexts[0].invoke_result, "");
        assert_eq!(log_contexts[0].instruction_index, 0);
        assert_eq!(log_contexts[0].data_logs[0], "f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");

        let program_data = LogContext::get_program_data("Program data: f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");
        assert_eq!(program_data, "f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");
        let not_data = LogContext::get_program_data(
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD invoke [1]",
        );
        assert!(not_data.is_empty());
    }

    #[test]
    fn log_context_json_test() {
        let raw_logs: Vec<String> = vec![
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD invoke [1]",
            "Program log: Instruction: PlacePerpOrderV3",
            "Program log: GetOraclePrice Pyth price: price=18211908 age=0",
            "Program data: f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA",
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD consumed 36432 of 1015220 compute units",
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD success"
        ].into_iter().map(|s| s.to_string()).collect();

        let programs_selector = ProgramsSelector::new_all_programs();

        let log_contexts = LogContext::parse_logs(
            &raw_logs,
            "".to_string(),
            &programs_selector,
            2523,
            "32432432".to_string(),
        );

        assert_eq!(log_contexts.len(), 1);

        let log_context_json = log_contexts[0].to_json();

        let log_context_from_json: LogContext = serde_json::from_str(&log_context_json).unwrap();

        assert_eq!(
            log_context_from_json.program_id,
            "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD"
        );
        assert_eq!(log_context_from_json.log_messages.len(), 2);
        assert_eq!(log_context_from_json.errors.len(), 0);
        assert_eq!(log_context_from_json.transaction_error, "".to_string());
        assert_eq!(log_context_from_json.data_logs.len(), 1);
        assert_eq!(log_context_from_json.raw_logs.len(), 6);
        assert_eq!(log_context_from_json.slot, 2523);
        assert_eq!(log_context_from_json.signature, "32432432");
        assert_eq!(log_context_from_json.parent_program_id, "");
        assert_eq!(log_context_from_json.depth, 1);
        // assert_eq!(log_context_from_json.id, 0);
        assert_eq!(log_context_from_json.invoke_result, "");
        assert_eq!(log_context_from_json.instruction_index, 0);
        assert_eq!(log_context_from_json.data_logs[0], "f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");

        let program_data = LogContext::get_program_data("Program data: f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");
        assert_eq!(program_data, "f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA");
        let not_data = LogContext::get_program_data(
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD invoke [1]",
        );
        assert!(not_data.is_empty());
    }

    #[test]
    fn log_parser_unmatched_test() {
        let raw_logs: Vec<String> = vec![
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD invoke [1]",
            "Transfer: insufficient lamports 5628503, need 6799920",
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD consumed 36432 of 1015220 compute units",
            "Program ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD success"
        ].into_iter().map(|s| s.to_string()).collect();

        let programs_selector = ProgramsSelector::new_all_programs();

        let log_contexts = LogContext::parse_logs(
            &raw_logs,
            "".to_string(),
            &programs_selector,
            2523,
            "32432432".to_string(),
        );

        assert_eq!(
            log_contexts[0].program_id,
            "ZETAxsqBRek56DhiGXrn75yj2NHU3aYUnxvHXpkf3aD"
        );
        assert_eq!(log_contexts[0].log_messages.len(), 0);
        assert_eq!(log_contexts[0].errors.len(), 0);
        assert_eq!(log_contexts[0].transaction_error, "".to_string());
        assert_eq!(log_contexts[0].data_logs.len(), 0);
        assert_eq!(log_contexts[0].raw_logs.len(), 4);
        assert_eq!(log_contexts[0].slot, 2523);
        assert_eq!(log_contexts[0].signature, "32432432");
        assert_eq!(log_contexts[0].parent_program_id, "");
        assert_eq!(log_contexts[0].depth, 1);
        // assert_eq!(log_contexts[0].id, 0);
        assert_eq!(log_contexts[0].invoke_result, "");
        assert_eq!(log_contexts[0].instruction_index, 0);
        assert_eq!(
            log_contexts[0].raw_logs[1],
            "Transfer: insufficient lamports 5628503, need 6799920"
        );
    }

    #[test]
    fn log_parser_transfer_test() {
        let raw_logs: Vec<String> = vec![
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR invoke [1]",
            "Program log: Instruction: OpenPosition",
            "Program 11111111111111111111111111111111 invoke [2]",
            "Program 11111111111111111111111111111111 success",
            "Program 11111111111111111111111111111111 invoke [2]",
            "Program 11111111111111111111111111111111 success",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
            "Program log: Instruction: InitializeMint",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2968 of 375840 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]",
            "Program log: Create",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
            "Program log: Instruction: GetAccountDataSize",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1622 of 358620 compute units",
            "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program 11111111111111111111111111111111 invoke [3]",
            "Program 11111111111111111111111111111111 success",
            "Program log: Initialize the associated token account",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
            "Program log: Instruction: InitializeImmutableOwner",
            "Program log: Please upgrade to SPL Token 2022 for immutable owner support",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 352130 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
            "Program log: Instruction: InitializeAccount3",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4241 of 348248 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20293 of 364017 compute units",
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
            "Program log: Instruction: MintTo",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4538 of 327259 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s invoke [2]",
            "Program log: IX: Create Metadata Accounts v3",
            "Program 11111111111111111111111111111111 invoke [3]",
            "Transfer: insufficient lamports 13792320, need 15616720",
            "Program 11111111111111111111111111111111 failed: custom program error: 0x1",
            "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s consumed 8635 of 318403 compute units",
            "Program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s failed: custom program error: 0x1",
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR consumed 90232 of 400000 compute units",
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR failed: custom program error: 0x1"
        ].into_iter().map(|s| s.to_string()).collect();

        let programs_selector = ProgramsSelector::new(&["*".to_string()]);

        let log_contexts = LogContext::parse_logs(&raw_logs, "".to_string(), &programs_selector,216778028, "KDhFgTogstghe9P1jVjVepnwfR9ZbcU8a6D21jXBh3PPyfkkd92MmevsWW7qb6QtfmfmWxAPYnL3xZR81xVCmeQ".to_string());

        assert_eq!(log_contexts.len(), 12);
    }

    #[test]
    fn test_logs_with_json_as_string() {
        let raw_logs: Vec<String> = vec![  "Program ComputeBudget111111111111111111111111111111 invoke [1]",
               "Program ComputeBudget111111111111111111111111111111 success",
               "Program ComputeBudget111111111111111111111111111111 invoke [1]",
               "Program ComputeBudget111111111111111111111111111111 success",
               "Program JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB invoke [1]",
               "Program log: Instruction: Route",
               "Program DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1 invoke [2]",
               "Program log: Instruction: Swap",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
               "Program log: Instruction: Transfer",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 263590 compute units",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
               "Program log: Instruction: Transfer",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4736 of 255983 compute units",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
               "Program DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1 consumed 30825 of 281191 compute units",
               "Program DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1 success",
               "Program data: UWzjvs3QCsS9Lo1QvAknjl4Uv2VimkiW9YGeysQPUMhLdXhHupRk3Mb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hoIYBAAAAAAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAVswTgAAAAAA",
               "Program 2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c invoke [2]",
               "Program log: Instruction: Swap",
               "Program log: AMM: {\"p\":86eq4kdBkUCHGdCC2SfcqGHRCBGhp2M89aCmuvvxaXsm}",
               "Program log: Oracle: {\"a\":1951643950.4947462,\"b\":983844291,\"c\":3600000000000,\"d\":1952402379}",
               "Program log: Amount: {\"in\":5124187,\"out\":99965,\"impact\":0.04}",
               "Program log: TotalFee: {\"fee\":2049,\"percent\":0.04}",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
               "Program log: Instruction: Transfer",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4736 of 160952 compute units",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
               "Program log: Instruction: MintTo",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 153343 compute units",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
               "Program log: Instruction: Transfer",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 145991 compute units",
               "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
               "Program 2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c consumed 94705 of 231343 compute units",
               "Program 2wT8Yq49kHgDzXuPxZSaeLaH1qbmGXtEyPy64bL7aD3c success",
               "Program data: UWzjvs3QCsQczpiYNW3rPyw0jcqiQE9VjpDsNcrjOdrGVQQtZANXrwabiFf+q4GE+2h/Y0YYwDXaxDncGus7VZig8AAAAAABWzBOAAAAAADG+nrzvtutOj1l82qryXQxsbvkwtL24OR8pgIDRS9dYX2GAQAAAAAA",
               "Program log: AnchorError occurred. Error Code: SlippageToleranceExceeded. Error Number: 6001. Error Message: Slippage tolerance exceeded.",
               "Program JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB consumed 167887 of 300000 compute units",
               "Program JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB failed: custom program error: 0x1771"
        ].into_iter().map(|s| s.to_string()).collect();

        let programs_selector = ProgramsSelector::new(&["*".to_string()]);

        let log_contexts = LogContext::parse_logs(&raw_logs, "".to_string(), &programs_selector,216778028, "KDhFgTogstghe9P1jVjVepnwfR9ZbcU8a6D21jXBh3PPyfkkd92MmevsWW7qb6QtfmfmWxAPYnL3xZR81xVCmeQ".to_string());
    }
}
