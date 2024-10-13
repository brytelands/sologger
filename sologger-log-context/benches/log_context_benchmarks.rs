use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;

fn generate_sample_logs(n: usize) -> Vec<String> {
    let mut logs = Vec::with_capacity(n);
    for i in 0..n {
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
    }
    logs
}

fn bench_parse_logs(c: &mut Criterion) {
    let logs = generate_sample_logs(100);
    let programs_selector = ProgramsSelector::new(&["*".to_string()]);

    c.bench_function("parse_logs 100 entries", |b| {
        b.iter(|| {
            LogContext::parse_logs(
                black_box(&logs),
                black_box("".to_string()),
                black_box(&programs_selector),
                black_box(1),
                black_box("12345".to_string()),
            )
        })
    });
}

fn bench_get_invoke_program_id(c: &mut Criterion) {
    let log = "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string();

    c.bench_function("get_invoke_program_id", |b| {
        b.iter(|| LogContext::get_invoke_program_id(black_box(&log)))
    });
}

fn bench_get_program_data(c: &mut Criterion) {
    let log = "Program data: f8oPt8jABAy1K0GKz0oSSO8oves0qt09GsKz1QNA3hkOpcvC0rPMywt4KffaIJMAVQlyjQhUVOXGyn09Lxu29Ty1k5m72ijBAAAAAAAAAAAAuZINBxwAAAC7AAAAAAAAAIjnFQEAAAAA".to_string();

    c.bench_function("get_program_data", |b| {
        b.iter(|| LogContext::get_program_data(black_box(&log)))
    });
}

fn bench_to_json(c: &mut Criterion) {
    let log_context = LogContext::new(
        "9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7".to_string(),
        1,
        "unique_id".to_string(),
        0,
        1,
        "signature".to_string(),
    );

    c.bench_function("to_json", |b| {
        b.iter(|| black_box(&log_context).to_json())
    });
}

criterion_group!(
    benches,
    bench_parse_logs,
    bench_get_invoke_program_id,
    bench_get_program_data,
    bench_to_json
);
criterion_main!(benches);