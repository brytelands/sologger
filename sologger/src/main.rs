use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use log::trace;

use sologger::log_subscriber;
use sologger::logger_lib::init_logger;
use sologger::sologger_config::SologgerConfig;
use sologger_idl_decoder::IdlRegistry;
use sologger_log_context::programs_selector::ProgramsSelector;

#[tokio::main]
async fn main() -> Result<()> {
    let (sologger_config, program_selector, idl_registry) =
        load_config().expect("Error loading sologger config");
    init_logger(&sologger_config);

    #[cfg(not(target_os = "windows"))]
    match spawn_signal_handler() {
        Ok(handler) => handler,
        Err(_) => panic!("Can't init signal handler"),
    };

    log_subscriber::start_client(&sologger_config, &program_selector, &idl_registry)
        .await
        .expect("Error starting WebSocket for log subscription");

    Ok(())
}

fn load_config() -> Result<(SologgerConfig, ProgramsSelector, IdlRegistry)> {
    let args: Vec<String> = env::args().collect();
    let sologger_config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        env::var("SOLOGGER_APP_CONFIG_LOC")
            .unwrap_or("./config/local/sologger-config.json".to_string())
    };

    trace!("sologger_config_path: {}", sologger_config_path);
    let mut file = File::open(Path::new(sologger_config_path.as_str()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read contents of sologger-config.json");

    let result: serde_json::Value = serde_json::from_str(&contents).unwrap();
    trace!("SologgerConfig: {}", result.to_string());
    let programs_selector = create_programs_selector_from_config(&result);
    let idl_registry = create_idl_registry_from_config(&result);
    let sologger_config = serde_json::from_str(&contents).map_err(|_err| ConfigError::Loading)?;

    Ok((sologger_config, programs_selector, idl_registry))
}

/// Builds the IDL registry from the optional "idls" map in sologger-config.json:
/// program ID -> path of an Anchor IDL JSON file (either spec version), relative to the
/// working directory. A missing or unreadable IDL is reported and skipped rather than
/// aborting startup, since decoding is an enrichment on top of normal parsing.
fn create_idl_registry_from_config(config: &serde_json::Value) -> IdlRegistry {
    let mut registry = IdlRegistry::new();
    let Some(idls) = config["idls"].as_object() else {
        return registry;
    };

    for (program_id, path_value) in idls {
        let Some(path) = path_value.as_str() else {
            // eprintln because the logger is not initialized until after config loading
            eprintln!(
                "sologger: idls entry for {} is not a string path, skipping",
                program_id
            );
            continue;
        };
        match std::fs::read_to_string(path) {
            Ok(idl_json) => match registry.insert_json(program_id, &idl_json) {
                Ok(()) => trace!("loaded IDL for {} from {}", program_id, path),
                Err(err) => eprintln!(
                    "sologger: failed to parse IDL for {} from {}: {}",
                    program_id, path, err
                ),
            },
            Err(err) => eprintln!(
                "sologger: failed to read IDL for {} from {}: {}",
                program_id, path, err
            ),
        }
    }
    registry
}

fn create_programs_selector_from_config(config: &serde_json::Value) -> ProgramsSelector {
    let programs_selector = &config["programsSelector"];

    if programs_selector.is_null() {
        ProgramsSelector::default()
    } else {
        let programs = &programs_selector["programs"];
        let programs: Vec<String> = if programs.is_array() {
            programs
                .as_array()
                .unwrap()
                .iter()
                .map(|val| val.as_str().unwrap().to_string())
                .collect()
        } else {
            Vec::default()
        };

        ProgramsSelector::new(&programs)
    }
}

#[cfg(not(target_os = "windows"))]
fn spawn_signal_handler() -> Result<()> {
    let mut signals = signal_hook::iterator::Signals::new([
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ])?;

    std::thread::spawn(move || {
        let mut stop_in_progress = false;
        for _sig in signals.forever() {
            std::thread::spawn(move || {
                log::logger().flush();
                signal_hook::low_level::exit(0)
            });
            if stop_in_progress {
                signal_hook::low_level::exit(1)
            }
            stop_in_progress = true;
        }
    });
    Ok(())
}

#[derive(Debug)]
enum ConfigError {
    Loading,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ConfigError::*;
        match self {
            Loading => write!(f, "Loading"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_programs_selector_from_config_null() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com"
        });

        let programs_selector = create_programs_selector_from_config(&config);
        assert!(!programs_selector.select_all_programs);
        assert!(programs_selector.programs.is_empty());
    }

    #[test]
    fn test_create_programs_selector_from_config_with_programs() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector": {
                "programs": [
                    "11111111111111111111111111111112",
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                ]
            }
        });

        let programs_selector = create_programs_selector_from_config(&config);
        assert!(!programs_selector.select_all_programs);
        assert_eq!(programs_selector.programs.len(), 2);
        assert!(programs_selector.is_program_selected_string("11111111111111111111111111111112"));
        assert!(programs_selector
            .is_program_selected_string("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));
    }

    #[test]
    fn test_create_programs_selector_from_config_all_programs() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector": {
                "programs": ["*"]
            }
        });

        let programs_selector = create_programs_selector_from_config(&config);
        assert!(programs_selector.select_all_programs);
        assert_eq!(programs_selector.programs.len(), 0);
    }

    #[test]
    fn test_create_programs_selector_from_config_empty_programs() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector": {
                "programs": []
            }
        });

        let programs_selector = create_programs_selector_from_config(&config);
        assert!(!programs_selector.select_all_programs);
        assert!(programs_selector.programs.is_empty());
    }

    #[test]
    fn test_create_programs_selector_from_config_non_array_programs() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "programsSelector": {
                "programs": "not_an_array"
            }
        });

        let programs_selector = create_programs_selector_from_config(&config);
        assert!(!programs_selector.select_all_programs);
        assert!(programs_selector.programs.is_empty());
    }

    #[test]
    fn test_create_idl_registry_from_config_missing() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com"
        });

        let registry = create_idl_registry_from_config(&config);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_create_idl_registry_from_config_with_idl() {
        // Uses the real Raydium IDL fixture from the decoder crate
        let idl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../sologger-idl-decoder/tests/fixtures/raydium_cp_swap_idl.json"
        );
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "idls": {
                "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C": idl_path
            }
        });

        let registry = create_idl_registry_from_config(&config);
        assert_eq!(registry.len(), 1);
        assert!(registry
            .get("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C")
            .is_some());
    }

    #[test]
    fn test_create_idl_registry_from_config_bad_entries() {
        let config = json!({
            "rpcUrl": "wss://api.mainnet-beta.solana.com",
            "idls": {
                "MissingFile111111111111111111111111111111111": "./no/such/file.json",
                "NotAString1111111111111111111111111111111111": 42
            }
        });

        // Bad entries are skipped with a warning instead of aborting startup
        let registry = create_idl_registry_from_config(&config);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::Loading;
        assert_eq!(format!("{}", error), "Loading");
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::Loading;
        assert_eq!(format!("{:?}", error), "Loading");
    }
}
