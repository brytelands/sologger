#[cfg(any(feature = "enable_logstash", feature = "enable_otel"))]
use std::path::Path;
#[cfg(feature = "enable_otel")]
use sologger_log_transport::opentelemetry_lib::get_otel_config;

use crate::sologger_config::SologgerConfig;

/// Initializes whichever transports are compiled in and configured. When none end up
/// active — no transport features, or their config files are absent — falls back to
/// pretty console output instead of panicking, so a bare `cargo run` against a local
/// validator just works.
pub fn init_logger(sologger_config: &SologgerConfig) {
    #[allow(unused_mut)]
    let mut transport_active = false;

    #[cfg(feature = "enable_logstash")]
    if init_logger_logstash(sologger_config) {
        transport_active = true;
    }
    #[cfg(feature = "enable_otel")]
    if init_logger_otel(sologger_config) {
        transport_active = true;
    }

    if !transport_active {
        crate::console_logger::enable();
        eprintln!("sologger: no transport configured — using pretty console output");
    }
}

/// Returns true when the logstash transport was initialized. A missing or unset log4rs
/// config disables the transport (with a notice) rather than panicking.
#[cfg(feature = "enable_logstash")]
pub fn init_logger_logstash(sologger_config: &SologgerConfig) -> bool {
    let location = &sologger_config.log4rs_config_location;
    if location.is_empty() || !Path::new(location).exists() {
        eprintln!(
            "sologger: log4rs config '{}' not found — logstash transport disabled",
            location
        );
        return false;
    }
    sologger_log_transport::logstash_lib::init_logstash_logger(location)
        .expect("Logger not initialized");
    true
}

/// Returns true when the OpenTelemetry transport was initialized. A missing or unset
/// config disables the transport (with a notice) rather than panicking.
#[cfg(feature = "enable_otel")]
pub fn init_logger_otel(sologger_config: &SologgerConfig) -> bool {
    let location = &sologger_config.opentelemetry_config_location;
    if location.is_empty() || !Path::new(location).exists() {
        eprintln!(
            "sologger: opentelemetry config '{}' not found — otel transport disabled",
            location
        );
        return false;
    }

    let config = get_otel_config(location);
    let _ = sologger_log_transport::opentelemetry_lib::init_logs_opentelemetry(&config);

    if config.enable_traces {
        match sologger_log_transport::opentelemetry_lib::init_tracer(&config) {
            Ok(_provider) => crate::telemetry::enable_traces(),
            Err(err) => eprintln!("sologger: failed to initialize OTel tracer: {}", err),
        }
    }
    if config.enable_metrics {
        match sologger_log_transport::opentelemetry_lib::init_metrics(&config) {
            Ok(_provider) => crate::telemetry::enable_metrics(),
            Err(err) => eprintln!("sologger: failed to initialize OTel metrics: {}", err),
        }
    }
    true
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
