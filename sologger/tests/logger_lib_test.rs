#[cfg(test)]
mod tests {
    use serde_json::json;
    use sologger::logger_lib::{init_logger};
    use sologger::sologger_config::SologgerConfig;

    #[tokio::test]
    pub async fn init_logger_test() {
        let config = json!(
            {
                "log4rsConfigLocation": "./tests/config/log4rs-config.yml",
                "opentelemetryConfigLocation": "./tests/config/opentelemetry-config.json",
                "rpcUrl": "wss://api.mainnet-beta.solana.com",
                "programsSelector" : {
                    "programs" : ["*"]
                }
            }
        );
        println!("current dir: {:?}", std::env::current_dir());
        let sologger_config = serde_json::from_value::<SologgerConfig>(config).unwrap();
        init_logger(&sologger_config);
    }
}