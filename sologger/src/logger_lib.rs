#[cfg(any(feature = "enable_logstash", feature = "enable_otel"))]
use log::{LevelFilter, Log};
#[cfg(any(feature = "enable_logstash", feature = "enable_otel"))]
use sologger_log_transport::fanout::FanoutLogger;
#[cfg(feature = "enable_logstash")]
use sologger_log_transport::logstash_lib::{build_logstash_logger, LogstashLogger};
#[cfg(feature = "enable_otel")]
use sologger_log_transport::opentelemetry_lib::{
    build_logs_opentelemetry, init_metrics, init_tracer, load_otel_config, OtelLogBridge,
    OtelProviders,
};
#[cfg(any(feature = "enable_logstash", feature = "enable_otel"))]
use std::path::Path;
#[cfg(feature = "enable_otel")]
use std::sync::OnceLock;
use std::time::Duration;

use crate::sologger_config::SologgerConfig;

/// Providers created by `build_otel`, kept so `shutdown_transports` can flush them.
#[cfg(feature = "enable_otel")]
static OTEL_PROVIDERS: OnceLock<OtelProviders> = OnceLock::new();

/// Initializes whichever transports are compiled in and configured. Each transport
/// builds its own logger, and they are installed together behind one [`FanoutLogger`]
/// (the `log` crate allows a single process-wide logger), so logstash and OpenTelemetry
/// can run in the same process. When none end up active — no transport features, or
/// their config files are absent or invalid — falls back to pretty console output.
#[cfg_attr(
    not(any(feature = "enable_logstash", feature = "enable_otel")),
    allow(unused_variables)
)]
pub fn init_logger(sologger_config: &SologgerConfig) {
    #[cfg(any(feature = "enable_logstash", feature = "enable_otel"))]
    {
        let mut loggers: Vec<Box<dyn Log>> = Vec::new();
        // The level every transport other than logstash needs
        #[allow(unused_mut)]
        let mut others_level = LevelFilter::Off;

        #[cfg(feature = "enable_otel")]
        if let Some(bridge) = build_otel(sologger_config) {
            others_level = others_level.max(bridge.level());
            loggers.push(Box::new(bridge));
        }

        #[allow(unused_mut)]
        let mut max_level = others_level;
        #[cfg(feature = "enable_logstash")]
        if let Some(logstash) = build_logstash(sologger_config) {
            // A log4rs reload resets the process-wide max level to log4rs's own; restore
            // what the other transports need as well
            let handle = logstash.handle();
            logstash.watch_config(move || {
                log::set_max_level(handle.max_log_level().max(others_level));
            });
            max_level = max_level.max(logstash.max_level());
            loggers.push(Box::new(logstash.into_logger()));
        }

        if !loggers.is_empty() {
            log::set_max_level(max_level);
            if log::set_boxed_logger(Box::new(FanoutLogger::new(loggers))).is_err() {
                eprintln!("sologger: a logger was already installed; transports not attached");
            }
            return;
        }
    }

    crate::console_logger::enable();
    eprintln!("sologger: no transport configured — using pretty console output");
}

/// The logstash transport's logger, or `None` (with a notice) when its log4rs config is
/// unset, missing or invalid.
#[cfg(feature = "enable_logstash")]
fn build_logstash(sologger_config: &SologgerConfig) -> Option<LogstashLogger> {
    let location = &sologger_config.log4rs_config_location;
    if location.is_empty() || !Path::new(location).exists() {
        eprintln!(
            "sologger: log4rs config '{}' not found — logstash transport disabled",
            location
        );
        return None;
    }
    match build_logstash_logger(location) {
        Ok(logstash) => Some(logstash),
        Err(err) => {
            eprintln!(
                "sologger: invalid log4rs config '{}': {:#} — logstash transport disabled",
                location, err
            );
            None
        }
    }
}

/// The OpenTelemetry log bridge, or `None` (with a notice) when its config is unset or
/// missing, or the exporter cannot be built. Also starts trace/metric export when enabled.
#[cfg(feature = "enable_otel")]
fn build_otel(sologger_config: &SologgerConfig) -> Option<OtelLogBridge> {
    let location = &sologger_config.opentelemetry_config_location;
    if location.is_empty() || !Path::new(location).exists() {
        eprintln!(
            "sologger: opentelemetry config '{}' not found — otel transport disabled",
            location
        );
        return None;
    }

    let config = match load_otel_config(location) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("sologger: {:#} — otel transport disabled", err);
            return None;
        }
    };
    let mut providers = OtelProviders::default();
    let bridge = match build_logs_opentelemetry(&config) {
        Ok((provider, bridge)) => {
            providers.logs = Some(provider);
            bridge
        }
        Err(err) => {
            eprintln!(
                "sologger: failed to initialize the OTel log exporter for '{}': {} — otel transport disabled",
                config.endpoint, err
            );
            return None;
        }
    };

    if config.enable_traces {
        match init_tracer(&config) {
            Ok(provider) => {
                providers.traces = Some(provider);
                crate::telemetry::enable_traces();
            }
            Err(err) => eprintln!("sologger: failed to initialize OTel tracer: {}", err),
        }
    }
    if config.enable_metrics {
        match init_metrics(&config) {
            Ok(provider) => {
                providers.metrics = Some(provider);
                crate::telemetry::enable_metrics();
            }
            Err(err) => eprintln!("sologger: failed to initialize OTel metrics: {}", err),
        }
    }
    let _ = OTEL_PROVIDERS.set(providers);
    Some(bridge)
}

/// Delivers telemetry the transports are still buffering, then flushes the logger. Call
/// right before exiting. Bounded by `timeout` so an unreachable collector cannot hold up
/// shutdown (exporters retry with backoff).
#[cfg_attr(not(feature = "enable_otel"), allow(unused_variables))]
pub fn shutdown_transports(timeout: Duration) {
    #[cfg(feature = "enable_otel")]
    if let Some(providers) = OTEL_PROVIDERS.get().cloned() {
        let (done, finished) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = done.send(providers.shutdown());
        });
        match finished.recv_timeout(timeout) {
            Ok(errors) => {
                for error in errors {
                    eprintln!("sologger: OTel shutdown failed for {}", error);
                }
            }
            Err(_) => eprintln!(
                "sologger: OTel shutdown did not finish within {:?}; buffered telemetry may be lost",
                timeout
            ),
        }
    }
    log::logger().flush();
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
