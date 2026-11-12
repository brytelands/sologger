use anyhow::Result;
use log::kv::Value;
use log::{Level, Log, Record};

use sologger_log_context::sologger_log_context::LogContext;

pub async fn log_contexts_from_logs(log_contexts: &Vec<LogContext>) -> Result<()> {
    if crate::console_logger::is_enabled() {
        crate::console_logger::print_log_contexts(log_contexts);
        return Ok(());
    }
    for log_context in log_contexts {
        // The same cheap level check the `log` macros make before building a record
        if level_of(log_context) <= log::max_level() {
            log_log_context(log::logger(), log_context);
        }
    }
    Ok(())
}

fn level_of(log_context: &LogContext) -> Level {
    if log_context.has_errors() {
        Level::Error
    } else {
        Level::Info
    }
}

/// Logs one record to `logger`: the context's JSON as the message (what LogStash ships),
/// plus its fields as key-values, which the OpenTelemetry bridge exports as log record
/// attributes so a backend can filter on them. The names match the span attributes in
/// `solana_telemetry`, so a log record and its span can be correlated. Empty optional
/// fields are left out.
fn log_log_context(logger: &dyn Log, log_context: &LogContext) {
    let mut key_values: Vec<(&str, Value)> = vec![
        (
            "solana.signature",
            Value::from(log_context.signature.as_str()),
        ),
        ("solana.slot", Value::from(log_context.slot)),
        (
            "solana.program_id",
            Value::from(log_context.program_id.as_str()),
        ),
        ("solana.depth", Value::from(log_context.depth)),
        (
            "solana.instruction_index",
            Value::from(log_context.instruction_index),
        ),
        (
            "solana.compute_units.consumed",
            Value::from(log_context.consumed_cu),
        ),
        ("solana.compute_units.max", Value::from(log_context.max_cu)),
    ];
    let optional = [
        ("solana.parent_program_id", &log_context.parent_program_id),
        ("solana.instruction", &log_context.instruction_name),
        ("solana.invoke_result", &log_context.invoke_result),
        ("solana.transaction_error", &log_context.transaction_error),
    ];
    for (key, value) in optional {
        if !value.is_empty() {
            key_values.push((key, Value::from(value.as_str())));
        }
    }
    if let Some(code) = log_context.error_code {
        key_values.push(("solana.error_code", Value::from(code)));
    }
    if let Some(name) = &log_context.error_name {
        key_values.push(("solana.error_name", Value::from(name.as_str())));
    }

    logger.log(
        &Record::builder()
            .level(level_of(log_context))
            .target(module_path!())
            .module_path_static(Some(module_path!()))
            .file_static(Some(file!()))
            .line(Some(line!()))
            .key_values(&key_values)
            .args(format_args!("{}", log_context.to_json()))
            .build(),
    );
}

#[cfg(test)]
mod attribute_tests {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use log::kv::{Key, Value, VisitSource};
    use log::{Level, Log, Metadata, Record};

    use sologger_log_context::sologger_log_context::LogContext;

    use super::log_log_context;

    /// Records each record's level, message and key-values.
    #[derive(Default)]
    struct Capture {
        records: Mutex<Vec<(Level, String, BTreeMap<String, String>)>>,
    }

    impl Log for Capture {
        fn enabled(&self, _: &Metadata) -> bool {
            true
        }
        fn log(&self, record: &Record) {
            struct Collect(BTreeMap<String, String>);
            impl<'kvs> VisitSource<'kvs> for Collect {
                fn visit_pair(
                    &mut self,
                    key: Key<'kvs>,
                    value: Value<'kvs>,
                ) -> Result<(), log::kv::Error> {
                    self.0.insert(key.to_string(), value.to_string());
                    Ok(())
                }
            }
            let mut kvs = Collect(BTreeMap::new());
            record.key_values().visit(&mut kvs).unwrap();
            self.records
                .lock()
                .unwrap()
                .push((record.level(), record.args().to_string(), kvs.0));
        }
        fn flush(&self) {}
    }

    fn context() -> LogContext {
        LogContext {
            log_messages: vec![],
            data_logs: vec![],
            decoded_events: vec![],
            raw_logs: vec![],
            errors: vec![],
            error_code: None,
            error_name: None,
            transaction_error: "".to_string(),
            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            parent_program_id: "".to_string(),
            depth: 1,
            id: "".to_string(),
            instruction_index: 2,
            instruction_name: "".to_string(),
            invoke_result: "".to_string(),
            slot: 301_000_000,
            signature: "5sig".to_string(),
            consumed_cu: 4_645,
            max_cu: 200_000,
        }
    }

    #[test]
    fn records_carry_the_log_context_fields_as_key_values() {
        let capture = Capture::default();
        let log_context = context();

        log_log_context(&capture, &log_context);

        let records = capture.records.lock().unwrap();
        let (level, message, kvs) = &records[0];
        assert_eq!(*level, Level::Info);
        assert_eq!(
            *message,
            log_context.to_json(),
            "message must stay the JSON"
        );
        let expected: BTreeMap<String, String> = [
            ("solana.signature", "5sig"),
            ("solana.slot", "301000000"),
            (
                "solana.program_id",
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            ),
            ("solana.depth", "1"),
            ("solana.instruction_index", "2"),
            ("solana.compute_units.consumed", "4645"),
            ("solana.compute_units.max", "200000"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert_eq!(*kvs, expected, "empty optional fields must be left out");
    }

    #[test]
    fn failed_records_are_errors_with_the_error_fields() {
        let capture = Capture::default();
        let log_context = LogContext {
            parent_program_id: "11111111111111111111111111111111".to_string(),
            instruction_name: "Transfer".to_string(),
            invoke_result: "failed".to_string(),
            transaction_error: "InstructionError(0, Custom(1))".to_string(),
            error_code: Some(1),
            error_name: Some("InsufficientFunds".to_string()),
            ..context()
        };

        log_log_context(&capture, &log_context);

        let records = capture.records.lock().unwrap();
        let (level, _, kvs) = &records[0];
        assert_eq!(*level, Level::Error);
        for (key, value) in [
            (
                "solana.parent_program_id",
                "11111111111111111111111111111111",
            ),
            ("solana.instruction", "Transfer"),
            ("solana.invoke_result", "failed"),
            ("solana.transaction_error", "InstructionError(0, Custom(1))"),
            ("solana.error_code", "1"),
            ("solana.error_name", "InsufficientFunds"),
        ] {
            assert_eq!(kvs.get(key).map(String::as_str), Some(value), "{key}");
        }
    }
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
