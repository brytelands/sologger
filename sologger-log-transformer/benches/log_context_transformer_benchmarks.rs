use criterion::{black_box, criterion_group, criterion_main, Criterion};
use solana_rpc_client_api::response::{Response, RpcLogsResponse, RpcResponseContext};
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedTransaction, EncodedTransactionWithStatusMeta,
    TransactionDetails, UiConfirmedBlock, UiMessage, UiParsedMessage,
    UiTransaction, UiTransactionStatusMeta,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_transformer::log_context_transformer::{from_encoded_confirmed_block, from_rpc_logs_response, from_rpc_response, from_ui_confirmed_block};

fn create_mock_ui_confirmed_block() -> UiConfirmedBlock {
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

    UiConfirmedBlock {
        previous_blockhash: "".to_string(),
        blockhash: "".to_string(),
        parent_slot: 0,
        transactions: Some(vec![transaction]),
        signatures: None,
        rewards: None,
        num_reward_partitions: Some(1),
        block_time: None,
        block_height: None,
    }
}

fn create_mock_encoded_confirmed_block() -> EncodedConfirmedBlock {
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

    EncodedConfirmedBlock {
        previous_blockhash: "".to_string(),
        blockhash: "".to_string(),
        parent_slot: 0,
        transactions: vec![transaction],
        rewards: vec![],
        num_partitions: Some(1),
        block_time: None,
        block_height: None,
    }
}

fn create_mock_rpc_logs_response() -> RpcLogsResponse {
    RpcLogsResponse {
        signature: "pF5oPR8R4vJwU2KeQm8BAAGYcTiikZkpJAmP8TuuVztkL2K6wZhxVKy9t6jSCMSpMMD3VE6Qek1YL5JAFvuBLQw".to_string(),
        err: None,
        logs: vec!["Program 11111111111111111111111111111111 invoke [1]".to_string(), "Program 11111111111111111111111111111111 success".to_string()],
    }
}

fn bench_from_ui_confirmed_block(c: &mut Criterion) {
    let ui_confirmed_block = create_mock_ui_confirmed_block();
    let program_selector = ProgramsSelector::new_all_programs();

    c.bench_function("from_ui_confirmed_block", |b| {
        b.iter(|| {
            from_ui_confirmed_block(
                black_box(ui_confirmed_block.clone()),
                black_box(219907401),
                black_box(&program_selector),
            )
        })
    });
}

fn bench_from_rpc_logs_response(c: &mut Criterion) {
    let rpc_logs_response = create_mock_rpc_logs_response();
    let program_selector = ProgramsSelector::new_all_programs();

    c.bench_function("from_rpc_logs_response", |b| {
        b.iter(|| {
            from_rpc_logs_response(
                black_box(&rpc_logs_response),
                black_box(323432),
                black_box(&program_selector),
            )
        })
    });
}

fn bench_from_rpc_response(c: &mut Criterion) {
    let rpc_logs_response = create_mock_rpc_logs_response();
    let response = Response {
        context: RpcResponseContext {
            slot: 12324,
            api_version: None,
        },
        value: rpc_logs_response,
    };
    let program_selector = ProgramsSelector::new_all_programs();

    c.bench_function("from_rpc_response", |b| {
        b.iter(|| from_rpc_response(black_box(&response), black_box(&program_selector)))
    });
}

criterion_group!(
    benches,
    bench_from_ui_confirmed_block,
    bench_from_rpc_logs_response,
    bench_from_rpc_response
);
criterion_main!(benches);