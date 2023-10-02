use serde_derive::{Deserialize, Serialize};
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
    /// The URL of the RPC endpoint to connect to
    pub rpc_url: String,
    /// The measure of the network confirmation and stake levels on a particular block.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub commitment_level: Option<String>,
    /// Set to true to subscribe to all transactions, including simple vote transactions. Otherwise, subscribe to all transactions except for simple vote transactions
    #[serde(default)]
    pub all_with_votes: bool,
}

#[test]
pub fn test_default() {
    let config = SologgerConfig::default();
    assert_eq!(config.opentelemetry_config_location, "");
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
