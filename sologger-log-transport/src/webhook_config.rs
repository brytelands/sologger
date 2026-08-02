use serde::{Deserialize, Serialize};

/// Webhook transport configuration: where to POST matched records and which records
/// match. All rule fields are optional; an empty config (just `url`) forwards every
/// parsed record as raw LogContext JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    ///The webhook URL to POST to (Discord webhook URL, Slack incoming webhook URL, or any HTTP endpoint)
    pub url: String,
    ///Payload shape: "discord" ({"content": ...}), "slack" ({"text": ...}), or "json" (the raw LogContext record). Defaults to "json"
    #[serde(default)]
    pub format: WebhookFormat,
    ///Only send records that carry errors
    #[serde(default)]
    pub errors_only: bool,
    ///Program allowlist; empty means all programs
    #[serde(default)]
    pub programs: Vec<String>,
    ///Instruction-name allowlist (exact match against the parsed Anchor instruction name); empty means all instructions
    #[serde(default)]
    pub instructions: Vec<String>,
    ///HTTP request timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_timeout_ms() -> u64 {
    5000
}

// Hand-written so that programmatic construction gets the same timeout default as
// deserialization — a derived Default would silently produce a zero-duration timeout.
impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            format: WebhookFormat::default(),
            errors_only: false,
            programs: Vec::new(),
            instructions: Vec::new(),
            timeout_ms: default_timeout_ms(),
        }
    }
}

/// The payload shape POSTed to the webhook URL.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookFormat {
    Discord,
    Slack,
    #[default]
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserializes_full_config() {
        let config = json!({
            "url": "https://discord.com/api/webhooks/123/abc",
            "format": "discord",
            "errorsOnly": true,
            "programs": ["CLMM9tUoggJu2wagPkkqs9eFG4BWhVBZWkP1qv3Sp7tR"],
            "instructions": ["OpenPosition"],
            "timeoutMs": 2500
        });

        let config = serde_json::from_value::<WebhookConfig>(config).unwrap();
        assert_eq!(config.url, "https://discord.com/api/webhooks/123/abc");
        assert_eq!(config.format, WebhookFormat::Discord);
        assert!(config.errors_only);
        assert_eq!(config.programs.len(), 1);
        assert_eq!(config.instructions, vec!["OpenPosition"]);
        assert_eq!(config.timeout_ms, 2500);
    }

    #[test]
    fn minimal_config_gets_defaults() {
        let config = json!({ "url": "http://localhost:9000/hook" });
        let config = serde_json::from_value::<WebhookConfig>(config).unwrap();
        assert_eq!(config.format, WebhookFormat::Json);
        assert!(!config.errors_only);
        assert!(config.programs.is_empty());
        assert!(config.instructions.is_empty());
        assert_eq!(config.timeout_ms, 5000);
    }
}
