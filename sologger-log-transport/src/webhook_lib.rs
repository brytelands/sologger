//! Webhook transport: POSTs matched `LogContext` records to Discord, Slack, or any
//! HTTP endpoint. Matching rules (errors only, program allowlist, instruction match)
//! live in [`WebhookConfig`]; all rules must pass for a record to be sent.

use std::fs;
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result as AnyResult};
use sologger_log_context::sologger_log_context::LogContext;

use crate::webhook_config::{WebhookConfig, WebhookFormat};

/// Discord caps message content at 2000 characters.
const DISCORD_CONTENT_LIMIT: usize = 2000;
/// Slack recommends keeping text payloads under ~3000 characters per section.
const SLACK_TEXT_LIMIT: usize = 3000;

/// Returns the WebhookConfig struct converted from the webhook config json.
pub fn get_webhook_config(path: &String) -> AnyResult<WebhookConfig> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read webhook config at {}", path))?;
    let config: WebhookConfig = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse webhook config at {}", path))?;
    Ok(config)
}

/// A configured webhook destination with its matching rules and HTTP client.
#[derive(Debug, Clone)]
pub struct WebhookTransport {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl WebhookTransport {
    pub fn new(config: WebhookConfig) -> AnyResult<Self> {
        if config.url.is_empty() {
            anyhow::bail!("webhook config has no url");
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .context("failed to build webhook HTTP client")?;
        Ok(Self { config, client })
    }

    /// Loads the config file at `path` and builds the transport.
    pub fn from_config_path(path: &String) -> AnyResult<Self> {
        Self::new(get_webhook_config(path)?)
    }

    pub fn config(&self) -> &WebhookConfig {
        &self.config
    }

    /// True when a record passes every configured rule.
    pub fn matches(&self, log_context: &LogContext) -> bool {
        if self.config.errors_only && !log_context.has_errors() {
            return false;
        }
        if !self.config.programs.is_empty()
            && !self.config.programs.contains(&log_context.program_id)
        {
            return false;
        }
        if !self.config.instructions.is_empty()
            && !self
                .config
                .instructions
                .contains(&log_context.instruction_name)
        {
            return false;
        }
        true
    }

    /// Builds the POST body for one record, according to the configured format.
    pub fn build_payload(&self, log_context: &LogContext) -> String {
        match self.config.format {
            WebhookFormat::Json => log_context.to_json(),
            WebhookFormat::Discord => serde_json::json!({
                "content": truncate(&build_message(log_context), DISCORD_CONTENT_LIMIT)
            })
            .to_string(),
            WebhookFormat::Slack => serde_json::json!({
                "text": truncate(&build_message(log_context), SLACK_TEXT_LIMIT)
            })
            .to_string(),
        }
    }

    /// The payloads for every matching record in a parsed batch.
    pub fn matching_payloads(&self, log_contexts: &[LogContext]) -> Vec<String> {
        log_contexts
            .iter()
            .filter(|context| self.matches(context))
            .map(|context| self.build_payload(context))
            .collect()
    }

    /// POSTs one payload. The caller decides retry/backoff policy; this is one attempt
    /// bounded by the configured timeout.
    pub async fn send_payload(&self, payload: String) -> AnyResult<()> {
        let response = self
            .client
            .post(&self.config.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload)
            .send()
            .await
            .context("webhook request failed")?;
        if !response.status().is_success() {
            anyhow::bail!("webhook responded with status {}", response.status());
        }
        Ok(())
    }

    /// Filters, formats and sends a whole batch, logging failures instead of
    /// propagating them. Returns how many records were sent successfully.
    pub async fn send_all(&self, log_contexts: &[LogContext]) -> usize {
        let mut sent = 0;
        for payload in self.matching_payloads(log_contexts) {
            match self.send_payload(payload).await {
                Ok(()) => sent += 1,
                Err(err) => log::warn!("webhook delivery failed: {}", err),
            }
        }
        sent
    }
}

/// Human-readable summary of one record, used for the Discord and Slack formats.
fn build_message(log_context: &LogContext) -> String {
    let status = if log_context.has_errors() {
        "❌"
    } else {
        "✅"
    };
    let mut lines = Vec::new();

    let mut headline = format!("{} `{}`", status, log_context.program_id);
    if !log_context.instruction_name.is_empty() {
        headline.push_str(&format!(" **{}**", log_context.instruction_name));
    }
    lines.push(headline);
    lines.push(format!(
        "slot {} · sig `{}`",
        log_context.slot, log_context.signature
    ));

    for error in &log_context.errors {
        lines.push(format!("error: {}", error));
    }
    if let Some(name) = &log_context.error_name {
        lines.push(format!(
            "error name: {} (0x{:x})",
            name,
            log_context.error_code.unwrap_or_default()
        ));
    }
    if !log_context.transaction_error.is_empty() && log_context.transaction_error != "null" {
        lines.push(format!("tx error: {}", log_context.transaction_error));
    }
    for event in &log_context.decoded_events {
        lines.push(format!("event: {}", event));
    }
    if log_context.consumed_cu > 0 || log_context.max_cu > 0 {
        lines.push(format!(
            "CU: {}/{}",
            log_context.consumed_cu, log_context.max_cu
        ));
    }

    lines.join("\n")
}

/// Truncates on a char boundary, appending an ellipsis when content was dropped.
fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(limit.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use sologger_log_context::programs_selector::ProgramsSelector;
    use sologger_log_context::sologger_log_context::LogContext;

    use super::*;
    use crate::webhook_config::{WebhookConfig, WebhookFormat};

    /// Two top-level invocations: contexts[0] is a successful token Transfer,
    /// contexts[1] is a failed CLMM OpenPosition. (The failure comes last because a
    /// real transaction aborts at the first failed top-level instruction.)
    fn parse_fixture() -> Vec<LogContext> {
        let logs: Vec<String> = vec![
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
            "Program log: Instruction: Transfer",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 200000 compute units",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR invoke [1]",
            "Program log: Instruction: OpenPosition",
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

    fn transport(config: WebhookConfig) -> WebhookTransport {
        WebhookTransport::new(config).unwrap()
    }

    #[test]
    fn empty_url_is_rejected() {
        assert!(WebhookTransport::new(WebhookConfig::default()).is_err());
    }

    #[test]
    fn errors_only_rule() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            errors_only: true,
            ..Default::default()
        });
        let contexts = parse_fixture();
        assert!(!transport.matches(&contexts[0])); // successful token transfer
        assert!(transport.matches(&contexts[1])); // failed CLMM invocation
        assert_eq!(transport.matching_payloads(&contexts).len(), 1);
    }

    #[test]
    fn program_allowlist_rule() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            programs: vec!["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()],
            ..Default::default()
        });
        let contexts = parse_fixture();
        assert!(transport.matches(&contexts[0]));
        assert!(!transport.matches(&contexts[1]));
    }

    #[test]
    fn instruction_rule() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            instructions: vec!["OpenPosition".to_string()],
            ..Default::default()
        });
        let contexts = parse_fixture();
        assert!(!transport.matches(&contexts[0]));
        assert!(transport.matches(&contexts[1]));
    }

    #[test]
    fn rules_combine_with_and() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            errors_only: true,
            programs: vec!["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()],
            ..Default::default()
        });
        let contexts = parse_fixture();
        // Tokenkeg matches the allowlist but has no errors
        assert!(!transport.matches(&contexts[0]));
        // CLMM has errors but is not allowlisted
        assert!(!transport.matches(&contexts[1]));
    }

    #[test]
    fn json_payload_is_the_raw_record() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            ..Default::default()
        });
        let contexts = parse_fixture();
        let payload = transport.build_payload(&contexts[1]);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["program_id"], "CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR");
        assert_eq!(parsed["error_code"], 1);
    }

    #[test]
    fn discord_payload_wraps_content() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            format: WebhookFormat::Discord,
            ..Default::default()
        });
        let contexts = parse_fixture();
        let payload = transport.build_payload(&contexts[1]);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let content = parsed["content"].as_str().unwrap();
        assert!(content.starts_with("❌"));
        assert!(content.contains("OpenPosition"));
        assert!(content.contains("slot 42"));
        assert!(content.contains("custom program error: 0x1"));
        assert!(content.chars().count() <= 2000);
    }

    #[test]
    fn slack_payload_wraps_text() {
        let transport = transport(WebhookConfig {
            url: "http://localhost/hook".to_string(),
            format: WebhookFormat::Slack,
            ..Default::default()
        });
        let contexts = parse_fixture();
        let payload = transport.build_payload(&contexts[0]);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let text = parsed["text"].as_str().unwrap();
        assert!(text.starts_with("✅"));
        assert!(text.contains("Transfer"));
        assert!(text.contains("CU: 4645/200000"));
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let long = "ß".repeat(3000);
        let truncated = super::truncate(&long, 2000);
        assert_eq!(truncated.chars().count(), 2000);
        assert!(truncated.ends_with('…'));
    }
}
