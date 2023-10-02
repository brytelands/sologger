use anyhow::Result;
use log::{error, info};

use solana_log_context::solana_log_context::LogContext;

pub async fn log_contexts_from_logs(log_contexts: &Vec<LogContext>) -> Result<()> {
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
        raw_logs: vec![],
        errors: vec![],
        transaction_error: "".to_string(),
        program_id: "".to_string(),
        parent_program_id: "".to_string(),
        depth: 0,
        id: "".to_string(),
        instruction_index: 0,
        invoke_result: "".to_string(),
        slot: 0,
        signature: "".to_string(),
    };

    let log_context_error = LogContext {
        log_messages: vec![],
        data_logs: vec![],
        raw_logs: vec![],
        errors: vec![],
        transaction_error: "Error".to_string(),
        program_id: "".to_string(),
        parent_program_id: "".to_string(),
        depth: 0,
        id: "".to_string(),
        instruction_index: 0,
        invoke_result: "".to_string(),
        slot: 0,
        signature: "".to_string(),
    };

    let log_contexts = vec![log_context, log_context_error];
    let result = log_contexts_from_logs(&log_contexts);
    assert!(result.await.is_ok());
}