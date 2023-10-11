use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use log::{trace, Log};

use sologger_log_context::programs_selector::ProgramsSelector;
use sologger::log_subscriber;
use sologger::logger_lib::init_logger;
use sologger::sologger_config::SologgerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let (sologger_config, program_selector) = load_config().expect("Error loading sologger config");
    init_logger(&sologger_config);

    #[cfg(not(target_os = "windows"))]
    match spawn_signal_handler() {
        Ok(handler) => handler,
        Err(_) => panic!("Can't init signal handler"),
    };

    log_subscriber::start_client(&sologger_config, &program_selector)
        .await
        .expect("Error starting WebSocket for log subscription");

    Ok(())
}

fn load_config() -> Result<(SologgerConfig, ProgramsSelector)> {
    let args: Vec<String> = env::args().collect();
    let sologger_config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        env::var("SOLOGGER_APP_CONFIG_LOC").unwrap_or("./config/local/sologger-config.json".to_string())
    };

    trace!("sologger_config_path: {}", sologger_config_path);
    let mut file = File::open(Path::new(sologger_config_path.as_str()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read contents of sologger-config.json");

    let result: serde_json::Value = serde_json::from_str(&contents).unwrap();
    trace!("SologgerConfig: {}", result.to_string());
    let programs_selector = create_programs_selector_from_config(&result);
    let sologger_config = serde_json::from_str(&contents).map_err(|_err| ConfigError::Loading)?;

    Ok((sologger_config, programs_selector))
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
