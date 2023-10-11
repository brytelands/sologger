use sologger_log_transport::opentelemetry_lib::get_otel_config;
use std::path::Path;

use crate::sologger_config::SologgerConfig;

pub fn init_logger(sologger_config: &SologgerConfig) {
    #[cfg(feature = "enable_logstash")]
    init_logger_logstash(sologger_config);
    #[cfg(feature = "enable_otel")]
    init_logger_otel(sologger_config);
}

#[cfg(feature = "enable_logstash")]
pub fn init_logger_logstash(sologger_config: &SologgerConfig) {
    if !Path::new(&sologger_config.log4rs_config_location).exists() {
        panic!("Log4rs config file not found");
    };
    sologger_log_transport::logstash_lib::init_logstash_logger(
        &sologger_config.log4rs_config_location,
    )
    .expect("Logger not initialized");
}

#[cfg(feature = "enable_otel")]
pub fn init_logger_otel(sologger_config: &SologgerConfig) {
    let config = get_otel_config(&sologger_config.opentelemetry_config_location);
    let _ = sologger_log_transport::opentelemetry_lib::init_logs_opentelemetry(&config);
}

// #[cfg(test)]
// mod tests {
//     use crate::logger_lib::init_logger;
//     use crate::sologger_config::SologgerConfig;
//     use serde_json::json;
//
//     #[test]
//     pub fn init_logger_test() {
//         //TODO fix for config location
//         let config = json!(
//             {
//                 "log4rsConfigLocation": "./config/log4rs-config.yml",
//                 "rpcUrl": "wss://api.mainnet-beta.solana.com",
//                 "programsSelector" : {
//                     "programs" : ["*"]
//                 }
//             }
//         );
//
//         let sologger_config = serde_json::from_value::<SologgerConfig>(config).unwrap();
//         init_logger(&sologger_config);
//     }
// }
