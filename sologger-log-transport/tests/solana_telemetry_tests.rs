#![cfg(feature = "otel")]

use std::time::Duration;

use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::{SpanId, Status, TracerProvider};
use opentelemetry::{Key, Value};
use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider, SpanData};
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;
use sologger_log_transport::solana_telemetry::{
    record_transaction_trace_with_tracer, SologgerMetrics,
};

/// A realistic failing CPI transaction: CLMM -> (system, token, AToken -> ..., metaplex
/// -> system Transfer failure). Parses into 12 LogContexts.
fn failing_cpi_logs() -> Vec<String> {
    vec![
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
        "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR failed: custom program error: 0x1",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

fn parse(logs: &Vec<String>, signature: &str) -> Vec<LogContext> {
    LogContext::parse_logs(
        logs,
        "".to_string(),
        &ProgramsSelector::new_all_programs(),
        216778028,
        signature.to_string(),
    )
}

fn attr<'a>(span: &'a SpanData, key: &str) -> Option<&'a Value> {
    span.attributes
        .iter()
        .find(|kv| kv.key == Key::new(key.to_string()))
        .map(|kv| &kv.value)
}

fn duration(span: &SpanData) -> Duration {
    span.end_time.duration_since(span.start_time).unwrap()
}

#[test]
fn cpi_span_tree_test() {
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("test");

    let log_contexts = parse(&failing_cpi_logs(), "SIG_A");
    assert_eq!(log_contexts.len(), 12);

    record_transaction_trace_with_tracer(&tracer, &log_contexts);
    provider.force_flush().unwrap();

    let spans = exporter.get_finished_spans().unwrap();
    // 12 program invocations + 1 transaction root
    assert_eq!(spans.len(), 13);

    // Everything belongs to one trace
    let trace_id = spans[0].span_context.trace_id();
    assert!(spans.iter().all(|s| s.span_context.trace_id() == trace_id));

    let root = spans.iter().find(|s| s.name == "transaction").unwrap();
    assert_eq!(root.parent_span_id, SpanId::INVALID);
    assert_eq!(
        attr(root, "solana.signature").unwrap().as_str(),
        "SIG_A"
    );
    assert!(matches!(root.status, Status::Error { .. }));

    // The CLMM top-level invocation: named from Phase 1's instruction_name, child of root
    let clmm = spans
        .iter()
        .find(|s| s.name == "CLMM9tUo OpenPosition")
        .unwrap();
    assert_eq!(clmm.parent_span_id, root.span_context.span_id());
    assert!(matches!(clmm.status, Status::Error { .. }));
    assert_eq!(
        attr(clmm, "solana.error_code").unwrap(),
        &Value::I64(1)
    );
    // Synthetic duration: consumed CU rendered as microseconds
    assert_eq!(duration(clmm), Duration::from_micros(90232));

    // Metaplex has no Anchor-style instruction log, so its span is named by program id
    let metaplex = spans.iter().find(|s| s.name == "metaqbxx").unwrap();
    assert_eq!(metaplex.parent_span_id, clmm.span_context.span_id());

    // The deepest failure: the system program invocation under metaplex
    let transfer_failure = spans
        .iter()
        .find(|s| {
            s.parent_span_id == metaplex.span_context.span_id()
        })
        .unwrap();
    assert_eq!(
        attr(transfer_failure, "solana.program_id").unwrap().as_str(),
        "11111111111111111111111111111111"
    );
    assert_eq!(attr(transfer_failure, "solana.depth").unwrap(), &Value::I64(3));
    assert!(matches!(transfer_failure.status, Status::Error { .. }));

    // A successful CPI leaf keeps an Unset status and its own CU duration
    let mint_to = spans
        .iter()
        .find(|s| s.name == "Tokenkeg MintTo")
        .unwrap();
    assert_eq!(mint_to.status, Status::Unset);
    assert_eq!(duration(mint_to), Duration::from_micros(4538));
    assert_eq!(
        attr(mint_to, "solana.compute_units.max").unwrap(),
        &Value::I64(327259)
    );

    // Sibling spans are laid out sequentially inside their parent
    let get_size = spans
        .iter()
        .find(|s| s.name == "Tokenkeg GetAccountDataSize")
        .unwrap();
    let init_owner = spans
        .iter()
        .find(|s| s.name == "Tokenkeg InitializeImmutableOwner")
        .unwrap();
    assert!(get_size.end_time <= init_owner.start_time);
}

#[test]
fn one_trace_per_transaction_test() {
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("test");

    let mut batch = parse(&failing_cpi_logs(), "SIG_A");
    batch.extend(parse(&failing_cpi_logs(), "SIG_B"));

    record_transaction_trace_with_tracer(&tracer, &batch);
    provider.force_flush().unwrap();

    let spans = exporter.get_finished_spans().unwrap();
    assert_eq!(spans.len(), 26);

    let roots: Vec<_> = spans.iter().filter(|s| s.name == "transaction").collect();
    assert_eq!(roots.len(), 2);
    assert_ne!(
        roots[0].span_context.trace_id(),
        roots[1].span_context.trace_id()
    );
}

#[test]
fn empty_batch_is_a_noop_test() {
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("test");

    record_transaction_trace_with_tracer(&tracer, &[]);
    provider.force_flush().unwrap();
    assert!(exporter.get_finished_spans().unwrap().is_empty());
}

#[test]
fn metrics_recording_test() {
    let exporter = InMemoryMetricExporter::default();
    let provider = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter.clone()).build())
        .build();
    let metrics = SologgerMetrics::with_meter(&provider.meter("test"));

    let log_contexts = parse(&failing_cpi_logs(), "SIG_A");
    metrics.record(&log_contexts);
    metrics.record_reconnect();
    provider.force_flush().unwrap();

    let finished = exporter.get_finished_metrics().unwrap();
    let names: Vec<String> = finished
        .iter()
        .flat_map(|rm| rm.scope_metrics())
        .flat_map(|sm| sm.metrics())
        .map(|m| m.name().to_string())
        .collect();

    assert!(names.contains(&"sologger.compute_units".to_string()));
    assert!(names.contains(&"sologger.transactions".to_string()));
    assert!(names.contains(&"sologger.transactions.failed".to_string()));
    assert!(names.contains(&"sologger.websocket.reconnects".to_string()));
    // No truncated logs in this fixture, so the counter has no data points yet
    assert!(!names.contains(&"sologger.logs.truncated".to_string()));
}

#[test]
fn truncated_log_metric_test() {
    let exporter = InMemoryMetricExporter::default();
    let provider = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter.clone()).build())
        .build();
    let metrics = SologgerMetrics::with_meter(&provider.meter("test"));

    let logs: Vec<String> = vec![
        "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR invoke [1]",
        "Log truncated",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    let log_contexts = parse(&logs, "SIG_T");

    metrics.record(&log_contexts);
    provider.force_flush().unwrap();

    let finished = exporter.get_finished_metrics().unwrap();
    let names: Vec<String> = finished
        .iter()
        .flat_map(|rm| rm.scope_metrics())
        .flat_map(|sm| sm.metrics())
        .map(|m| m.name().to_string())
        .collect();
    assert!(names.contains(&"sologger.logs.truncated".to_string()));
}
