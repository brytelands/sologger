//! Pretty console output: the fallback "transport" when no LogStash or OpenTelemetry
//! config is present. Renders each transaction as a colored, CPI-indented tree, turning
//! sologger into an everyday `tail -f` for solana-test-validator.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

use sologger_log_context::sologger_log_context::LogContext;

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Switches log output to pretty console rendering. Called by `init_logger` when no
/// transport config is available.
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Prints a parsed batch to stdout, colorized when stdout is a terminal.
pub fn print_log_contexts(log_contexts: &[LogContext]) {
    let color = std::io::stdout().is_terminal();
    print!("{}", render_batch(log_contexts, color));
}

struct Palette {
    red: &'static str,
    green: &'static str,
    yellow: &'static str,
    cyan: &'static str,
    dim: &'static str,
    bold: &'static str,
    reset: &'static str,
}

const COLORS: Palette = Palette {
    red: "\x1b[31m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    cyan: "\x1b[36m",
    dim: "\x1b[2m",
    bold: "\x1b[1m",
    reset: "\x1b[0m",
};

const PLAIN: Palette = Palette {
    red: "",
    green: "",
    yellow: "",
    cyan: "",
    dim: "",
    bold: "",
    reset: "",
};

/// Renders a parsed batch as human-readable text. Contexts are grouped by signature;
/// each transaction gets a header line and a depth-indented line per program invocation.
pub fn render_batch(log_contexts: &[LogContext], color: bool) -> String {
    let p = if color { &COLORS } else { &PLAIN };
    let mut out = String::new();

    let mut start = 0;
    for end in 1..=log_contexts.len() {
        if end == log_contexts.len()
            || log_contexts[end].signature != log_contexts[start].signature
        {
            render_transaction(&mut out, &log_contexts[start..end], p);
            start = end;
        }
    }
    out
}

fn render_transaction(out: &mut String, contexts: &[LogContext], p: &Palette) {
    let Some(first) = contexts.first() else {
        return;
    };

    let failed = contexts.iter().any(|c| c.has_errors());
    let verdict = if failed {
        format!("{}✗ FAILED{}", p.red, p.reset)
    } else {
        format!("{}✓{}", p.green, p.reset)
    };
    out.push_str(&format!(
        "{}── slot {} · {}{} {}\n",
        p.bold, first.slot, first.signature, p.reset, verdict
    ));
    if !first.transaction_error.is_empty() && first.transaction_error != "null" {
        out.push_str(&format!(
            "  {}tx error: {}{}\n",
            p.red, first.transaction_error, p.reset
        ));
    }

    for context in contexts {
        render_invocation(out, context, p);
    }
}

fn render_invocation(out: &mut String, context: &LogContext, p: &Palette) {
    let indent = "  ".repeat(context.depth.max(1));

    let mut line = format!("{}{}{}{}", indent, p.bold, short_id(&context.program_id), p.reset);
    if !context.instruction_name.is_empty() {
        line.push_str(&format!(" {}{}{}", p.cyan, context.instruction_name, p.reset));
    }
    if context.consumed_cu > 0 || context.max_cu > 0 {
        line.push_str(&format!(
            " {}{}/{} CU{}",
            p.dim, context.consumed_cu, context.max_cu, p.reset
        ));
    }
    if context.errors.is_empty() {
        line.push_str(&format!(" {}✓{}", p.green, p.reset));
    } else {
        line.push_str(&format!(" {}✗{}", p.red, p.reset));
    }
    out.push_str(&line);
    out.push('\n');

    for message in &context.log_messages {
        // The instruction name already appears on the invocation line
        if message.strip_prefix("Instruction: ") == Some(context.instruction_name.as_str()) {
            continue;
        }
        out.push_str(&format!("{}  {}· {}{}\n", indent, p.dim, message, p.reset));
    }
    for event in &context.decoded_events {
        out.push_str(&format!("{}  {}★ {}{}\n", indent, p.yellow, event, p.reset));
    }
    for error in &context.errors {
        out.push_str(&format!("{}  {}✗ {}{}\n", indent, p.red, error, p.reset));
    }
    if let Some(name) = &context.error_name {
        let code = context.error_code.unwrap_or_default();
        out.push_str(&format!(
            "{}  {}✗ {} (0x{:x}){}\n",
            indent, p.red, name, code, p.reset
        ));
    }
    if !context.invoke_result.is_empty() {
        out.push_str(&format!(
            "{}  {}→ {}{}\n",
            indent, p.dim, context.invoke_result, p.reset
        ));
    }
}

fn short_id(id: &str) -> String {
    if id.len() > 9 {
        format!("{}…", &id[..8])
    } else {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use sologger_log_context::programs_selector::ProgramsSelector;
    use sologger_log_context::sologger_log_context::LogContext;

    use super::render_batch;

    fn parse_fixture() -> Vec<LogContext> {
        let logs: Vec<String> = vec![
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR invoke [1]",
            "Program log: Instruction: OpenPosition",
            "Program log: some detail",
            "Program 11111111111111111111111111111111 invoke [2]",
            "Transfer: insufficient lamports 13792320, need 15616720",
            "Program 11111111111111111111111111111111 failed: custom program error: 0x1",
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR consumed 90232 of 400000 compute units",
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR failed: custom program error: 0x1",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        LogContext::parse_logs(
            &logs,
            "".to_string(),
            &ProgramsSelector::new_all_programs(),
            42,
            "TESTSIG".to_string(),
        )
    }

    #[test]
    fn renders_transaction_tree_without_color() {
        let contexts = parse_fixture();
        let output = render_batch(&contexts, false);

        assert!(output.contains("── slot 42 · TESTSIG ✗ FAILED"));
        // Top-level invocation at depth 1, instruction name on the line
        assert!(output.contains("\n  CLMM9tUo… OpenPosition 90232/400000 CU ✗"));
        // CPI child indented one level deeper
        assert!(output.contains("\n    11111111… ✗"));
        // System-program diagnostic captured as an error line under the child
        assert!(output.contains("      ✗ Transfer: insufficient lamports"));
        // Plain log message shown dim-less in no-color mode
        assert!(output.contains("    · some detail"));
        // The duplicate "Instruction: ..." message is suppressed
        assert!(!output.contains("· Instruction: OpenPosition"));
        // No ANSI escapes when color is off
        assert!(!output.contains('\x1b'));
    }

    #[test]
    fn renders_ansi_when_colored() {
        let contexts = parse_fixture();
        let output = render_batch(&contexts, true);
        assert!(output.contains("\x1b[31m")); // red for the failure
        assert!(output.contains("\x1b[0m"));
    }

    #[test]
    fn groups_by_signature() {
        let mut contexts = parse_fixture();
        let mut second = parse_fixture();
        for context in &mut second {
            context.signature = "OTHERSIG".to_string();
        }
        contexts.extend(second);

        let output = render_batch(&contexts, false);
        assert!(output.contains("── slot 42 · TESTSIG"));
        assert!(output.contains("── slot 42 · OTHERSIG"));
    }

    #[test]
    fn successful_transaction_gets_checkmark() {
        let logs: Vec<String> = vec![
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
            "Program log: Instruction: Transfer",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 200000 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &ProgramsSelector::new_all_programs(),
            7,
            "OKSIG".to_string(),
        );

        let output = render_batch(&contexts, false);
        assert!(output.contains("── slot 7 · OKSIG ✓"));
        assert!(output.contains("Tokenkeg… Transfer 4645/200000 CU ✓"));
    }
}
