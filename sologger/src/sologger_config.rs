use serde_derive::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;

/// This is the main configuration file for sologger. The location of this file is specified by the `SOLOGGER_APP_CONFIG_LOC` environment variable or as the first argument via the cargo run command.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SologgerConfig {
    /// The location of the log4rs config file
    #[serde(default)]
    pub log4rs_config_location: String,
    /// The location of the opentelemetry config file
    #[serde(default)]
    pub opentelemetry_config_location: String,
    /// The location of the webhook config file (used by binaries built with `enable_webhook`)
    #[serde(default)]
    pub webhook_config_location: String,
    /// The URL of the RPC endpoint to connect to
    pub rpc_url: String,
    /// The HTTP RPC endpoint used for getTransaction / getSignaturesForAddress calls
    /// (truncation backfill and historical backfill). When empty, it is derived from
    /// `rpcUrl`: ws(s):// becomes http(s)://, and port 8900 becomes 8899 (the local
    /// validator convention).
    #[serde(default)]
    pub rpc_http_url: String,
    /// The subscription used as the log source: "logsSubscribe" (default) or
    /// "blockSubscribe". blockSubscribe is not enabled on every RPC provider.
    #[serde(default)]
    pub source: LogSource,
    /// When a transaction's logs arrive truncated ("Log truncated"), refetch the full
    /// transaction over HTTP and re-parse it. On by default.
    #[serde(default = "default_true")]
    pub backfill_truncated: bool,
    /// Optional historical backfill run at startup: replays past transactions of the
    /// selected programs via getSignaturesForAddress + getTransaction.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backfill: Option<BackfillConfig>,
    /// The measure of the network confirmation and stake levels on a particular block.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub commitment_level: Option<String>,
    /// Set to true to subscribe to all transactions, including simple vote transactions. Otherwise, subscribe to all transactions except for simple vote transactions
    #[serde(default)]
    pub all_with_votes: bool,
}

fn default_true() -> bool {
    true
}

/// Which WebSocket subscription feeds the parser.
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogSource {
    #[default]
    #[serde(rename = "logsSubscribe")]
    LogsSubscribe,
    #[serde(rename = "blockSubscribe")]
    BlockSubscribe,
}

/// Historical backfill: replay past transactions of the selected programs through the
/// normal parsing/enrichment/export pipeline. Requires an explicit `programsSelector`
/// (getSignaturesForAddress needs concrete addresses).
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BackfillConfig {
    /// Only replay transactions at or after this slot
    #[serde(default)]
    pub from_slot: Option<u64>,
    /// Only replay transactions at or before this slot
    #[serde(default)]
    pub until_slot: Option<u64>,
    /// Maximum number of signatures to replay per program
    #[serde(default = "default_backfill_limit")]
    pub limit: usize,
    /// Delay between getTransaction calls, to respect RPC rate limits
    #[serde(default = "default_backfill_throttle_ms")]
    pub throttle_ms: u64,
    /// Exit once the backfill finishes instead of continuing into the live tail
    /// (post-mortem mode)
    #[serde(default)]
    pub exit_after: bool,
}

fn default_backfill_limit() -> usize {
    1000
}

fn default_backfill_throttle_ms() -> u64 {
    200
}

// Hand-written so programmatic construction gets the same limits as deserialization.
impl Default for BackfillConfig {
    fn default() -> Self {
        Self {
            from_slot: None,
            until_slot: None,
            limit: default_backfill_limit(),
            throttle_ms: default_backfill_throttle_ms(),
            exit_after: false,
        }
    }
}

impl SologgerConfig {
    /// The HTTP RPC endpoint: the configured `rpcHttpUrl`, or one derived from the
    /// WebSocket url (ws→http, wss→https, port 8900→8899).
    pub fn http_url(&self) -> String {
        if !self.rpc_http_url.is_empty() {
            return self.rpc_http_url.clone();
        }
        derive_http_url(&self.rpc_url)
    }
}

/// Derives an HTTP RPC url from a WebSocket url. Solana's local validator convention
/// puts the WebSocket on RPC port + 1, hence the 8900 → 8899 rewrite.
pub fn derive_http_url(ws_url: &str) -> String {
    let http = if let Some(rest) = ws_url.strip_prefix("wss://") {
        format!("https://{}", rest)
    } else if let Some(rest) = ws_url.strip_prefix("ws://") {
        format!("http://{}", rest)
    } else {
        ws_url.to_string()
    };
    http.replace(":8900", ":8899")
}

#[test]
pub fn test_default() {
    let config = SologgerConfig::default();
    assert_eq!(config.opentelemetry_config_location, "");
    assert_eq!(config.source, LogSource::LogsSubscribe);
    assert!(config.backfill.is_none());
}

#[test]
pub fn test_derive_http_url() {
    assert_eq!(
        derive_http_url("wss://api.mainnet-beta.solana.com"),
        "https://api.mainnet-beta.solana.com"
    );
    assert_eq!(
        derive_http_url("ws://127.0.0.1:8900"),
        "http://127.0.0.1:8899"
    );
    assert_eq!(
        derive_http_url("https://already-http.example.com"),
        "https://already-http.example.com"
    );

    let mut config = SologgerConfig {
        rpc_url: "wss://api.devnet.solana.com".to_string(),
        ..Default::default()
    };
    assert_eq!(config.http_url(), "https://api.devnet.solana.com");
    config.rpc_http_url = "https://my-rpc.example.com".to_string();
    assert_eq!(config.http_url(), "https://my-rpc.example.com");
}

#[test]
pub fn test_deserialize_ingestion_options() {
    let config = json!(
        {
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "rpcHttpUrl": "https://api.mainnet-beta.solana.com",
            "source": "blockSubscribe",
            "backfillTruncated": false,
            "backfill": {
                "fromSlot": 1000,
                "untilSlot": 2000,
                "limit": 50,
                "throttleMs": 100,
                "exitAfter": true
            }
        }
    );

    let sologger_config = serde_json::from_value::<SologgerConfig>(config).unwrap();
    assert_eq!(sologger_config.source, LogSource::BlockSubscribe);
    assert!(!sologger_config.backfill_truncated);
    assert_eq!(sologger_config.rpc_http_url, "https://api.mainnet-beta.solana.com");
    let backfill = sologger_config.backfill.unwrap();
    assert_eq!(backfill.from_slot, Some(1000));
    assert_eq!(backfill.until_slot, Some(2000));
    assert_eq!(backfill.limit, 50);
    assert_eq!(backfill.throttle_ms, 100);
    assert!(backfill.exit_after);
}

#[test]
pub fn test_backfill_defaults() {
    let backfill = serde_json::from_value::<BackfillConfig>(json!({})).unwrap();
    assert_eq!(backfill.limit, 1000);
    assert_eq!(backfill.throttle_ms, 200);
    assert!(!backfill.exit_after);
    assert_eq!(backfill, BackfillConfig::default());
}

#[test]
pub fn test_deserialize() {
    let config = json!(
        {
            "log4rsConfigLocation": "./config/log4rs-config.yml",
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector" : {
                "programs" : ["*"]
            }
        }
    );

    let sologger_config = serde_json::from_value::<SologgerConfig>(config).unwrap();
    assert_eq!(sologger_config.rpc_url, "wss://api.mainnet-beta.solana.com");
    assert_eq!(
        sologger_config.log4rs_config_location,
        "./config/log4rs-config.yml"
    );
    assert_eq!(sologger_config.all_with_votes, false);
    assert_eq!(sologger_config.commitment_level, None);
}

#[test]
pub fn test_deserialize_all() {
    let config = json!(
        {
            "log4rsConfigLocation": "./config/log4rs-config.yml",
            "opentelemetryConfigLocation": "./config/opentelemetry-config.json",
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector" : {
                "programs" : ["*"]
            },
            "allWithVotes": true,
            "commitmentLevel": "recent"
        }
    );

    let sologger_config = serde_json::from_value::<SologgerConfig>(config).unwrap();
    assert_eq!(sologger_config.rpc_url, "wss://api.mainnet-beta.solana.com");
    assert_eq!(
        sologger_config.log4rs_config_location,
        "./config/log4rs-config.yml"
    );
    assert_eq!(
        sologger_config.opentelemetry_config_location,
        "./config/opentelemetry-config.json"
    );
    assert_eq!(sologger_config.all_with_votes, true);
    assert_eq!(sologger_config.commitment_level.unwrap(), "recent");
}
