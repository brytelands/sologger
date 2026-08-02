use anyhow::Result;
use log::{error, info};

use sologger_log_context::sologger_log_context::LogContext;

pub async fn log_contexts_from_logs(log_contexts: &Vec<LogContext>) -> Result<()> {
    if crate::console_logger::is_enabled() {
        crate::console_logger::print_log_contexts(log_contexts);
        return Ok(());
    }
    for log_context in log_contexts {
        if log_context.has_errors() {
            error!("{}", &log_context.to_json());
        } else {
            info!("{}", &log_context.to_json());
        }
    }
    Ok(())
}

#[tokio::test]
pub async fn log_contexts_from_logs_test() {
    let log_context = LogContext {
        log_messages: vec![],
        data_logs: vec![],
        decoded_events: vec![],
        raw_logs: vec![],
        errors: vec![],
        error_code: None,
        error_name: None,
        transaction_error: "".to_string(),
        program_id: "".to_string(),
        parent_program_id: "".to_string(),
        depth: 0,
        id: "".to_string(),
        instruction_index: 0,
        instruction_name: "".to_string(),
        invoke_result: "".to_string(),
        slot: 0,
        signature: "".to_string(),
        consumed_cu: 0,
        max_cu: 0,
    };

    let log_context_error = LogContext {
        log_messages: vec![],
        data_logs: vec![],
        decoded_events: vec![],
        raw_logs: vec![],
        errors: vec![],
        error_code: None,
        error_name: None,
        transaction_error: "Error".to_string(),
        program_id: "".to_string(),
        parent_program_id: "".to_string(),
        depth: 0,
        id: "".to_string(),
        instruction_index: 0,
        instruction_name: "".to_string(),
        invoke_result: "".to_string(),
        slot: 0,
        signature: "".to_string(),
        consumed_cu: 0,
        max_cu: 0,
    };

    let log_contexts = vec![log_context, log_context_error];
    let result = log_contexts_from_logs(&log_contexts);
    assert!(result.await.is_ok());
}
