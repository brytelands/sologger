use solana_rpc_client_api::response::{Response, RpcLogsResponse};


use solana_transaction_status::{
    ConfirmedBlock, EncodedConfirmedBlock,
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
    EncodedTransactionWithStatusMeta, TransactionWithStatusMeta, UiConfirmedBlock,
    VersionedConfirmedBlock, VersionedTransactionWithStatusMeta,
};

use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;

/// Extracts log messages from a VersionedConfirmedBlock and returns a vector of LogContexts
pub fn from_version_confirmed_block(
    block: VersionedConfirmedBlock,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    block.transactions.iter().for_each(|tx| {
        let result = from_versioned_transaction(tx, slot, program_selector)
            .expect("Error processing logs for block");
        for log in result {
            block_log_contexts.push(log);
        }
    });
    Ok(block_log_contexts)
}

/// Extracts log messages from a ConfirmedBlock and returns a vector of LogContexts
pub fn from_confirmed_block(
    block: ConfirmedBlock,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    block.transactions.iter().for_each(|tx| {
        let result =
            from_transaction(tx, slot, program_selector).expect("Error processing logs for block");
        for log in result {
            block_log_contexts.push(log);
        }
    });
    Ok(block_log_contexts)
}

/// Extracts log messages from a EncodedConfirmedBlock and returns a vector of LogContexts
pub fn from_encoded_confirmed_block(
    block: EncodedConfirmedBlock,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    block.transactions.iter().for_each(|tx| {
        let result = from_encoded_transaction(tx, slot, program_selector)
            .expect("Error processing logs for block");
        for log in result {
            block_log_contexts.push(log);
        }
    });
    Ok(block_log_contexts)
}

/// Extracts log messages from a UiConfirmedBlock and returns a vector of LogContexts
pub fn from_ui_confirmed_block(
    block: UiConfirmedBlock,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    block.transactions.unwrap().iter().for_each(|tx| {
        let result = from_encoded_transaction(tx, slot, program_selector)
            .expect("Error processing logs for block");
        for log in result {
            block_log_contexts.push(log);
        }
    });
    Ok(block_log_contexts)
}

/// Extracts log messages from a EncodedTransactionWithStatusMeta and returns a vector of LogContexts
pub fn from_encoded_transaction(
    tx: &EncodedTransactionWithStatusMeta,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    let logs: Option<Vec<String>> = tx.meta.to_owned().unwrap().log_messages.into();
    let logs = logs.unwrap_or(vec![]);
    let te = tx.meta.to_owned().unwrap().err;
    let transaction_error = match te {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let inner_tx = &tx.transaction;
    let signature = match inner_tx {
        EncodedTransaction::Binary(_, _) => {
            tx.transaction.decode().unwrap().signatures[0].to_string()
        }
        EncodedTransaction::Json(inner_tx) => inner_tx.signatures[0].to_string(),
        _ => "".to_string(),
    };

    // let signature = tx.transaction.decode().unwrap().signatures[0];
    let log_contexts = LogContext::parse_logs(
        &logs,
        transaction_error,
        program_selector,
        slot,
        signature.to_string(),
    );
    for sologger_log_context in log_contexts {
        block_log_contexts.push(sologger_log_context);
    }
    Ok(block_log_contexts)
}

/// Extracts log messages from a EncodedConfirmedTransactionWithStatusMeta and returns a vector of LogContexts
pub fn from_encoded_confirmed_transaction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    let logs: Option<Vec<String>> = tx.transaction.meta.to_owned().unwrap().log_messages.into();
    let logs = logs.unwrap_or(vec![]);
    let te = tx.transaction.to_owned().meta.unwrap().err;
    let transaction_error = match te {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let inner_tx = &tx.transaction.transaction;
    let signature = match inner_tx {
        EncodedTransaction::Binary(_, _) => {
            tx.transaction.transaction.decode().unwrap().signatures[0].to_string()
        }
        EncodedTransaction::Json(inner_tx) => inner_tx.signatures[0].to_string(),
        _ => "".to_string(),
    };

    let log_contexts =
        LogContext::parse_logs(&logs, transaction_error, program_selector, slot, signature);
    for sologger_log_context in log_contexts {
        block_log_contexts.push(sologger_log_context);
    }
    Ok(block_log_contexts)
}

/// Extracts log messages from a TransactionWithStatusMeta and returns a vector of LogContexts
pub fn from_transaction(
    tx: &TransactionWithStatusMeta,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    let logs: Option<Vec<String>> = tx.get_status_meta().unwrap().log_messages;
    let logs = logs.unwrap_or(vec![]);
    let te = tx.get_status_meta().to_owned().unwrap().status.err();
    let transaction_error = match te {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };
    let signature = tx.transaction_signature().to_string();
    let log_contexts = LogContext::parse_logs(
        &logs,
        transaction_error,
        program_selector,
        slot,
        signature.to_string(),
    );
    for sologger_log_context in log_contexts {
        block_log_contexts.push(sologger_log_context);
    }
    Ok(block_log_contexts)
}

/// Extracts log messages from a VersionedTransactionWithStatusMeta and returns a vector of LogContexts
pub fn from_versioned_transaction(
    tx: &VersionedTransactionWithStatusMeta,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let mut block_log_contexts = Vec::new();
    let logs: Option<Vec<String>> = tx.meta.to_owned().log_messages;
    let logs = logs.unwrap_or(vec![]);
    let te = tx.meta.to_owned().status.err();
    let transaction_error = match te {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };
    let signature = tx.transaction.signatures[0];
    let log_contexts = LogContext::parse_logs(
        &logs,
        transaction_error,
        program_selector,
        slot,
        signature.to_string(),
    );
    for sologger_log_context in log_contexts {
        block_log_contexts.push(sologger_log_context);
    }
    Ok(block_log_contexts)
}

/// Extracts log messages from a Response<RpcLogsResponse> and returns a vector of LogContexts
pub fn from_rpc_response(
    response: &Response<RpcLogsResponse>,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let transaction_error = match response.value.err.clone() {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let sig = response.value.signature.to_string();
    let log_contexts = LogContext::parse_logs(
        &response.value.logs,
        transaction_error,
        program_selector,
        response.context.slot,
        sig,
    );

    Ok(log_contexts)
}

/// Extracts log messages from a RpcLogsResponse and returns a vector of LogContexts
pub fn from_rpc_logs_response(
    rpc_logs_response: &RpcLogsResponse,
    slot: u64,
    program_selector: &ProgramsSelector,
) -> anyhow::Result<Vec<LogContext>> {
    let transaction_error = match rpc_logs_response.err.clone() {
        None => "".to_string(),
        Some(err) => {
            format!("{}", err)
        }
    };

    let sig = rpc_logs_response.signature.to_string();
    let log_contexts = LogContext::parse_logs(
        &rpc_logs_response.logs,
        transaction_error,
        program_selector,
        slot,
        sig,
    );

    Ok(log_contexts)
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use solana_rpc_client::rpc_client::RpcClient;
    use solana_rpc_client_api::config::{RpcBlockConfig, RpcTransactionConfig};
    use solana_rpc_client_api::response::{Response, RpcLogsResponse, RpcResponseContext};
    use solana_sdk::clock::UnixTimestamp;
    use solana_sdk::commitment_config::CommitmentConfig;
    use solana_sdk::instruction::InstructionError;
    use solana_sdk::message::Message;
    use solana_sdk::signature::{Keypair, Signature, Signer};
    use solana_sdk::system_transaction;
    use solana_sdk::transaction::{Transaction, TransactionError, VersionedTransaction};
    use solana_transaction_status::option_serializer::OptionSerializer;
    use solana_transaction_status::{ConfirmedBlock, EncodedConfirmedBlock, EncodedTransaction, EncodedTransactionWithStatusMeta, TransactionDetails, TransactionStatusMeta, UiConfirmedBlock, UiMessage, UiParsedMessage, UiRawMessage, UiTransaction, UiTransactionEncoding, UiTransactionStatusMeta, VersionedConfirmedBlock, VersionedTransactionWithStatusMeta};

    use crate::log_context_transformer::{from_confirmed_block, from_encoded_confirmed_block, from_encoded_confirmed_transaction, from_encoded_transaction, from_rpc_logs_response, from_rpc_response, from_ui_confirmed_block, from_version_confirmed_block};
    use sologger_log_context::programs_selector::ProgramsSelector;

    #[test]
    fn test_block() {
        let _slot = 219907401;
        let signature = Signature::new_unique();

        let ui_parsed_message = UiParsedMessage {
            account_keys: vec![],
            recent_blockhash: "".to_string(),
            instructions: vec![],
            address_table_lookups: None,
        };
        let ui_transaction = UiTransaction {
            signatures: vec![signature.to_string()],
            message: UiMessage::Parsed(ui_parsed_message),
        };

        let transaction_status_meta = UiTransactionStatusMeta {
            err: None,
            status: Ok(()),
            fee: 0,
            pre_balances: vec![],
            post_balances: vec![],
            inner_instructions: OptionSerializer::None,
            log_messages: OptionSerializer::Some(vec![
                "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                "Program 11111111111111111111111111111111 success".to_string(),
            ]),
            pre_token_balances: OptionSerializer::None,
            post_token_balances: OptionSerializer::None,
            rewards: OptionSerializer::None,
            loaded_addresses: OptionSerializer::None,
            return_data: OptionSerializer::None,
            compute_units_consumed: OptionSerializer::None,
        };
        let transaction = EncodedTransactionWithStatusMeta {
            transaction: EncodedTransaction::Json(ui_transaction),
            meta: Some(transaction_status_meta),
            version: None,
        };

        let ui_confirmed_block = UiConfirmedBlock {
            previous_blockhash: "".to_string(),
            blockhash: "".to_string(),
            parent_slot: 0,
            transactions: Some(vec![transaction]),
            signatures: None,
            rewards: None,
            num_reward_partitions: Some(1),
            block_time: None,
            block_height: None,
        };

        let result = from_ui_confirmed_block(
            ui_confirmed_block,
            219907401,
            &ProgramsSelector::new_all_programs(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_transaction() {
        let _slot = 219907401;
        let client = RpcClient::new_mock("succeeds".to_string());
        let _config = RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Base58),
            transaction_details: Some(TransactionDetails::Full),
            rewards: Some(true),
            commitment: None,
            max_supported_transaction_version: Some(0),
        };

        let sig = Signature::from_str("pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw").unwrap();
        let tx = client
            .get_transaction(&sig, UiTransactionEncoding::Base58)
            .unwrap();

        let _result =
            from_encoded_confirmed_transaction(&tx, tx.slot, &ProgramsSelector::new_all_programs())
                .unwrap();
    }

    #[test]
    fn test_transaction_config_mock() {
        let client = RpcClient::new_mock("succeeds".to_string());
        let alice = Keypair::new();
        let bob = Keypair::new();
        let lamports = 50;
        let latest_blockhash = client.get_latest_blockhash().unwrap();
        let tx = system_transaction::transfer(&alice, &bob.pubkey(), lamports, latest_blockhash);
        let signature = client.send_and_confirm_transaction(&tx).unwrap();
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Binary),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };

        let tx = client
            .get_transaction_with_config(&signature, config)
            .unwrap();

        let result =
            from_encoded_confirmed_transaction(&tx, tx.slot, &ProgramsSelector::new_all_programs())
                .unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    pub fn test_parse_rpc_logs_response() {
        let rpc_logs_response = RpcLogsResponse {
            signature: "pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw".to_string(),
            err: None,
            logs: vec!["Program 11111111111111111111111111111111 invoke [1]".to_string(), "Program 11111111111111111111111111111111 success".to_string()],
        };

        let logs_contexts = from_rpc_logs_response(
            &rpc_logs_response,
            323432,
            &ProgramsSelector::new_all_programs(),
        )
        .unwrap();

        assert_eq!(logs_contexts.len(), 1);
    }

    #[test]
    pub fn test_parse_rpc_response() {
        let rpc_logs_response = RpcLogsResponse {
            signature: "pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw".to_string(),
            err: None,
            logs: vec!["Program 11111111111111111111111111111111 invoke [1]".to_string(), "Program 11111111111111111111111111111111 success".to_string()],
        };

        let response = Response {
            context: RpcResponseContext {
                slot: 12324,
                api_version: None,
            },
            value: rpc_logs_response,
        };

        let logs_contexts =
            from_rpc_response(&response, &ProgramsSelector::new_all_programs()).unwrap();

        assert_eq!(logs_contexts.len(), 1);
    }

    #[test]
    pub fn test_versioned_confirmed_block() {
        let signature = Signature::new_unique();
        let versioned_transaction = VersionedTransaction {
            signatures: vec![signature],
            message: Default::default(),
        };
        let transaction_status_meta = TransactionStatusMeta {
            status: Ok(()),
            fee: 0,
            pre_balances: vec![],
            post_balances: vec![],
            inner_instructions: None,
            log_messages: Some(vec![
                "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                "Program 11111111111111111111111111111111 success".to_string(),
            ]),
            pre_token_balances: None,
            post_token_balances: None,
            rewards: None,
            loaded_addresses: Default::default(),
            return_data: None,
            compute_units_consumed: None,
        };
        let transaction_with_status_meta = VersionedTransactionWithStatusMeta {
            transaction: versioned_transaction,
            meta: transaction_status_meta,
        };

        let confirmed_block = VersionedConfirmedBlock {
            previous_blockhash: "".to_string(),
            blockhash: "".to_string(),
            parent_slot: 0,
            transactions: vec![transaction_with_status_meta],
            rewards: vec![],
            num_partitions: Some(1),
            block_time: Some(UnixTimestamp::default()),
            block_height: Some(100),
        };
        let logs_contexts = from_version_confirmed_block(
            confirmed_block,
            123,
            &ProgramsSelector::new_all_programs(),
        )
        .unwrap();
        assert_eq!(logs_contexts.len(), 1);
    }

    #[test]
    pub fn test_encoded_confirmed_block() {
        let signature = Signature::new_unique();

        let ui_parsed_message = UiParsedMessage {
            account_keys: vec![],
            recent_blockhash: "".to_string(),
            instructions: vec![],
            address_table_lookups: None,
        };
        let ui_transaction = UiTransaction {
            signatures: vec![signature.to_string()],
            message: UiMessage::Parsed(ui_parsed_message),
        };

        let transaction_status_meta = UiTransactionStatusMeta {
            err: None,
            status: Ok(()),
            fee: 0,
            pre_balances: vec![],
            post_balances: vec![],
            inner_instructions: OptionSerializer::None,
            log_messages: OptionSerializer::Some(vec![
                "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                "Program 11111111111111111111111111111111 success".to_string(),
            ]),
            pre_token_balances: OptionSerializer::None,
            post_token_balances: OptionSerializer::None,
            rewards: OptionSerializer::None,
            loaded_addresses: OptionSerializer::None,
            return_data: OptionSerializer::None,
            compute_units_consumed: OptionSerializer::None,
        };
        let transaction = EncodedTransactionWithStatusMeta {
            transaction: EncodedTransaction::Json(ui_transaction),
            meta: Some(transaction_status_meta),
            version: None,
        };

        let confirmed_block = EncodedConfirmedBlock {
            previous_blockhash: "".to_string(),
            blockhash: "".to_string(),
            parent_slot: 0,
            transactions: vec![transaction],
            rewards: vec![],
            num_partitions: Some(1),
            block_time: Some(UnixTimestamp::default()),
            block_height: Some(100),
        };
        let logs_contexts = from_encoded_confirmed_block(
            confirmed_block,
            123,
            &ProgramsSelector::new_all_programs(),
        )
        .unwrap();

        assert_eq!(logs_contexts.len(), 1);
    }

    // Test for error cases in from_encoded_transaction
    #[test]
    fn test_from_encoded_transaction_error_cases() {
        let signature = Signature::new_unique();
        let ui_raw_message = UiRawMessage {
            header: Default::default(),
            account_keys: vec![],
            recent_blockhash: "".to_string(),
            instructions: vec![],
            address_table_lookups: None,
        };
        let ui_transaction = UiTransaction {
            signatures: vec![signature.to_string()],
            message: UiMessage::Raw(ui_raw_message),
        };
        let transaction_status_meta = UiTransactionStatusMeta {
            err: Some(TransactionError::AccountNotFound),
            status: Err(TransactionError::AccountNotFound),
            fee: 0,
            pre_balances: vec![],
            post_balances: vec![],
            inner_instructions: OptionSerializer::None,
            log_messages: OptionSerializer::None,
            pre_token_balances: OptionSerializer::None,
            post_token_balances: OptionSerializer::None,
            rewards: OptionSerializer::None,
            loaded_addresses: OptionSerializer::None,
            return_data: OptionSerializer::None,
            compute_units_consumed: OptionSerializer::None,
        };
        let transaction = EncodedTransactionWithStatusMeta {
            transaction: EncodedTransaction::Json(ui_transaction),
            meta: Some(transaction_status_meta),
            version: None,
        };

        let result = from_encoded_transaction(
            &transaction,
            123,
            &ProgramsSelector::new_all_programs(),
        )
            .unwrap();

        assert_eq!(result.len(), 0);
        // You might want to add more assertions here to check the error handling
    }
    
}
