//! Turns parsed `LogContext` records into OpenTelemetry traces and metrics.
//!
//! **Traces:** one trace per transaction, one span per program invocation, parented by
//! CPI depth — a Jaeger/SigNoz waterfall of the call tree. The transaction signature is
//! recorded as the `solana.signature` attribute on the root span; correlate and search
//! by that attribute.
//!
//! **Span timing is synthetic.** Solana logs carry no timestamps, so spans start at
//! export time and each span's duration is its consumed compute units rendered as
//! microseconds (1 CU = 1µs, minimum 1µs). Sibling spans are laid out sequentially
//! inside their parent. Durations therefore show CU proportions, not wall time.

use std::time::{Duration, SystemTime};

use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::trace::{Span, SpanBuilder, SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use sologger_log_context::sologger_log_context::LogContext;

/// Instrumentation scope name used for the tracer and meter.
pub const SCOPE_NAME: &str = "sologger";

/// Records one trace per transaction found in `log_contexts`, using the globally
/// installed tracer provider (see `init_tracer`). Contexts are grouped by signature, so
/// a batch spanning several transactions produces several traces.
pub fn record_transaction_trace(log_contexts: &[LogContext]) {
    record_transaction_trace_with_tracer(&global::tracer(SCOPE_NAME), log_contexts);
}

/// Same as [`record_transaction_trace`], but with an explicit tracer.
pub fn record_transaction_trace_with_tracer<T>(tracer: &T, log_contexts: &[LogContext])
where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    for transaction in group_by_signature(log_contexts) {
        record_single_transaction(tracer, transaction);
    }
}

fn group_by_signature(log_contexts: &[LogContext]) -> Vec<&[LogContext]> {
    let mut groups: Vec<&[LogContext]> = Vec::new();
    let mut start = 0;
    for i in 1..=log_contexts.len() {
        if i == log_contexts.len() || log_contexts[i].signature != log_contexts[start].signature {
            groups.push(&log_contexts[start..i]);
            start = i;
        }
    }
    groups
}

/// The CPI tree of one transaction, reconstructed from invoke order + depth.
struct InvocationNode {
    context_index: usize,
    children: Vec<InvocationNode>,
}

fn build_invocation_tree(log_contexts: &[LogContext]) -> Vec<InvocationNode> {
    let mut roots: Vec<InvocationNode> = Vec::new();
    // Stack of indices into the tree, one per open depth level
    let mut path: Vec<usize> = Vec::new();

    for (index, context) in log_contexts.iter().enumerate() {
        let depth = context.depth.max(1);
        path.truncate(depth - 1);

        let node = InvocationNode {
            context_index: index,
            children: Vec::new(),
        };
        // Walk down the current path to the insertion point
        let siblings = path.iter().fold(&mut roots, |level, &i| {
            &mut level[i].children
        });
        siblings.push(node);
        path.push(siblings.len() - 1);
    }
    roots
}

fn record_single_transaction<T>(tracer: &T, log_contexts: &[LogContext])
where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    let Some(first) = log_contexts.first() else {
        return;
    };

    let tree = build_invocation_tree(log_contexts);
    let start_time = SystemTime::now();

    let transaction_failed = !first.transaction_error.is_empty()
        || log_contexts.iter().any(|c| !c.errors.is_empty());

    let mut root_attributes = vec![
        KeyValue::new("solana.signature", first.signature.clone()),
        KeyValue::new("solana.slot", first.slot as i64),
    ];
    if !first.transaction_error.is_empty() && first.transaction_error != "null" {
        root_attributes.push(KeyValue::new(
            "solana.transaction_error",
            first.transaction_error.clone(),
        ));
    }

    let root_builder = SpanBuilder::from_name("transaction")
        .with_kind(SpanKind::Internal)
        .with_start_time(start_time)
        .with_attributes(root_attributes);
    let mut root_span = tracer.build_with_context(root_builder, &Context::new());
    if transaction_failed {
        root_span.set_status(Status::error("transaction failed"));
    }
    let root_context = Context::new().with_span(root_span);

    let mut cursor = start_time;
    for node in &tree {
        cursor = emit_span(tracer, &root_context, log_contexts, node, cursor);
    }

    root_context.span().end_with_timestamp(cursor);
}

/// Emits the span for one invocation and, recursively, its CPI children. Returns the
/// span's synthetic end time, which becomes the next sibling's start time.
fn emit_span<T>(
    tracer: &T,
    parent_context: &Context,
    log_contexts: &[LogContext],
    node: &InvocationNode,
    start_time: SystemTime,
) -> SystemTime
where
    T: Tracer,
    T::Span: Send + Sync + 'static,
{
    let context = &log_contexts[node.context_index];

    let builder = SpanBuilder::from_name(span_name(context))
        .with_kind(SpanKind::Internal)
        .with_start_time(start_time)
        .with_attributes(span_attributes(context));
    let mut span = tracer.build_with_context(builder, parent_context);
    if context.has_errors() {
        span.set_status(Status::error(context.errors.join("; ")));
    }
    let span_context = parent_context.with_span(span);

    let mut cursor = start_time;
    for child in &node.children {
        cursor = emit_span(tracer, &span_context, log_contexts, child, cursor);
    }

    // Synthetic duration: consumed CU as microseconds, never shorter than the children
    let own_end = start_time + Duration::from_micros(context.consumed_cu.max(1));
    let end_time = own_end.max(cursor);
    span_context.span().end_with_timestamp(end_time);
    end_time
}

fn span_name(context: &LogContext) -> String {
    let program = short_id(&context.program_id);
    if context.instruction_name.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, context.instruction_name)
    }
}

fn short_id(id: &str) -> &str {
    if id.len() > 8 { &id[..8] } else { id }
}

fn span_attributes(context: &LogContext) -> Vec<KeyValue> {
    let mut attributes = vec![
        KeyValue::new("solana.program_id", context.program_id.clone()),
        KeyValue::new("solana.depth", context.depth as i64),
        KeyValue::new("solana.instruction_index", context.instruction_index as i64),
        KeyValue::new("solana.compute_units.consumed", context.consumed_cu as i64),
        KeyValue::new("solana.compute_units.max", context.max_cu as i64),
    ];
    if !context.instruction_name.is_empty() {
        attributes.push(KeyValue::new(
            "solana.instruction",
            context.instruction_name.clone(),
        ));
    }
    if !context.parent_program_id.is_empty() {
        attributes.push(KeyValue::new(
            "solana.parent_program_id",
            context.parent_program_id.clone(),
        ));
    }
    if let Some(code) = context.error_code {
        attributes.push(KeyValue::new("solana.error_code", code as i64));
    }
    if let Some(name) = &context.error_name {
        attributes.push(KeyValue::new("solana.error_name", name.clone()));
    }
    if !context.invoke_result.is_empty() {
        attributes.push(KeyValue::new(
            "solana.invoke_result",
            context.invoke_result.clone(),
        ));
    }
    attributes
}

/// Metric instruments for parsed Solana logs. Create once (the instruments are cached
/// handles) and call [`SologgerMetrics::record`] per parsed batch.
pub struct SologgerMetrics {
    compute_units: Histogram<u64>,
    transactions: Counter<u64>,
    transaction_failures: Counter<u64>,
    truncated_logs: Counter<u64>,
    websocket_reconnects: Counter<u64>,
    slots_missed: Counter<u64>,
}

impl SologgerMetrics {
    /// Builds the instruments from the globally installed meter provider
    /// (see `init_metrics`).
    pub fn new() -> Self {
        Self::with_meter(&global::meter(SCOPE_NAME))
    }

    /// Builds the instruments from an explicit meter.
    pub fn with_meter(meter: &Meter) -> Self {
        Self {
            compute_units: meter
                .u64_histogram("sologger.compute_units")
                .with_unit("cu")
                .with_description("Compute units consumed per program invocation")
                .build(),
            transactions: meter
                .u64_counter("sologger.transactions")
                .with_description("Transactions processed")
                .build(),
            transaction_failures: meter
                .u64_counter("sologger.transactions.failed")
                .with_description("Transactions that failed, attributed to the deepest failing program")
                .build(),
            truncated_logs: meter
                .u64_counter("sologger.logs.truncated")
                .with_description("Transactions whose logs were truncated by the RPC")
                .build(),
            websocket_reconnects: meter
                .u64_counter("sologger.websocket.reconnects")
                .with_description("WebSocket subscription reconnects")
                .build(),
            slots_missed: meter
                .u64_counter("sologger.slots.missed")
                .with_description("Slots that passed while a subscription was disconnected")
                .build(),
        }
    }

    /// Records CU usage, failures and truncations for a parsed batch. Contexts are
    /// grouped by signature so transaction-level counters increment once per
    /// transaction.
    pub fn record(&self, log_contexts: &[LogContext]) {
        for context in log_contexts {
            if context.consumed_cu > 0 {
                let mut attributes =
                    vec![KeyValue::new("program_id", context.program_id.clone())];
                if !context.instruction_name.is_empty() {
                    attributes.push(KeyValue::new(
                        "instruction",
                        context.instruction_name.clone(),
                    ));
                }
                self.compute_units.record(context.consumed_cu, &attributes);
            }
        }

        for transaction in group_by_signature(log_contexts) {
            self.transactions.add(1, &[]);

            // Attribute the failure to the deepest failing invocation — the root cause
            // of the abort, not the outermost program that propagated it
            let failure_origin = transaction
                .iter()
                .filter(|c| c.has_errors())
                .max_by_key(|c| c.depth);
            if let Some(origin) = failure_origin {
                let mut attributes = vec![KeyValue::new("program_id", origin.program_id.clone())];
                if let Some(name) = &origin.error_name {
                    attributes.push(KeyValue::new("error_name", name.clone()));
                } else if let Some(code) = origin.error_code {
                    attributes.push(KeyValue::new("error_name", format!("0x{:x}", code)));
                }
                self.transaction_failures.add(1, &attributes);
            }

            if transaction
                .iter()
                .any(|c| c.invoke_result == "Log truncated")
            {
                self.truncated_logs.add(1, &[]);
            }
        }
    }

    /// Increments the WebSocket reconnect counter (used by the ingestion layer).
    pub fn record_reconnect(&self) {
        self.websocket_reconnects.add(1, &[]);
    }

    /// Records slots that passed while a subscription was disconnected — detected by
    /// comparing the last slot seen before a reconnect with the first one after.
    pub fn record_slot_gap(&self, missed_slots: u64) {
        self.slots_missed.add(missed_slots, &[]);
    }
}

impl Default for SologgerMetrics {
    fn default() -> Self {
        Self::new()
    }
}
