use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result as AnyResult};
use log::LevelFilter;
use log4rs::config::{load_config_file, Deserializers, RawConfig};
use log4rs::Handle;
use qoollo_log4rs_logstash::config::DeserializersExt;

/// A log4rs logger for the logstash transport, built from its config file but not yet
/// installed — so it can be installed alone or combined with other transports in a
/// [`crate::fanout::FanoutLogger`].
pub struct LogstashLogger {
    logger: log4rs::Logger,
    config_path: PathBuf,
    refresh_rate: Option<Duration>,
}

impl LogstashLogger {
    /// The most verbose level any logger in the config accepts.
    pub fn max_level(&self) -> LevelFilter {
        self.logger.max_log_level()
    }

    /// The config's `refresh_rate`: how often the file is checked for changes.
    pub fn refresh_rate(&self) -> Option<Duration> {
        self.refresh_rate
    }

    /// A handle to the logger's live configuration.
    pub fn handle(&self) -> Handle {
        self.logger.handle()
    }

    /// Reloads the config file when it changes, every `refresh_rate` (no-op without one),
    /// like log4rs's own `init_file`. `after_reload` runs after each applied change:
    /// log4rs resets the process-wide max level to its own on reload, so a combined
    /// logger uses it to restore the level the other transports need.
    pub fn watch_config<F>(&self, after_reload: F)
    where
        F: Fn() + Send + 'static,
    {
        let Some(refresh_rate) = self.refresh_rate else {
            return;
        };
        let path = self.config_path.clone();
        let handle = self.handle();
        let mut last = std::fs::read_to_string(&path).ok();
        std::thread::spawn(move || {
            let mut rate = refresh_rate;
            loop {
                std::thread::sleep(rate);
                let current = std::fs::read_to_string(&path).ok();
                if current.is_none() || current == last {
                    continue;
                }
                last = current;
                match load(&path) {
                    Ok((config, new_rate)) => {
                        handle.set_config(config);
                        after_reload();
                        match new_rate {
                            Some(new_rate) => rate = new_rate,
                            None => return,
                        }
                    }
                    Err(err) => eprintln!(
                        "sologger: keeping the previous log4rs config; reloading '{}' failed: {:#}",
                        path.display(),
                        err
                    ),
                }
            }
        });
    }

    /// The `log::Log` implementation, ready to install or combine.
    pub fn into_logger(self) -> log4rs::Logger {
        self.logger
    }
}

/// Builds the logstash transport's logger from a log4rs config file without installing
/// it. A missing or invalid file is an error.
pub fn build_logstash_logger(log4rs_config_location: &str) -> AnyResult<LogstashLogger> {
    let config_path = PathBuf::from(log4rs_config_location);
    let (config, refresh_rate) = load(&config_path)?;
    Ok(LogstashLogger {
        logger: log4rs::Logger::new(config),
        config_path,
        refresh_rate,
    })
}

/// Initializes the logstash transport as the process-wide logger, reloading its config
/// when it changes. Fails on a missing or invalid config, or when a logger is already
/// installed — to run alongside other transports, use [`build_logstash_logger`] with a
/// [`crate::fanout::FanoutLogger`] instead.
#[cfg(feature = "logstash")]
pub fn init_logstash_logger(log4rs_config_location: &str) -> AnyResult<()> {
    let logstash = build_logstash_logger(log4rs_config_location)?;
    logstash.watch_config(|| {});
    log::set_max_level(logstash.max_level());
    log::set_boxed_logger(Box::new(logstash.into_logger()))
        .context("a logger is already installed")?;
    Ok(())
}

/// Reads the config with the logstash appender registered, plus its refresh rate.
fn load(path: &Path) -> AnyResult<(log4rs::Config, Option<Duration>)> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("reading log4rs config '{}'", path.display()))?;
    let raw: RawConfig = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yml") | Some("yaml") => serde_yaml::from_str(&source)?,
        Some("json") => serde_json::from_str(&source)?,
        _ => anyhow::bail!(
            "log4rs config '{}' must be .yml, .yaml or .json",
            path.display()
        ),
    };
    let config = load_config_file(path, Deserializers::default().with_logstash())
        .with_context(|| format!("loading log4rs config '{}'", path.display()))?;
    Ok((config, raw.refresh_rate()))
}
