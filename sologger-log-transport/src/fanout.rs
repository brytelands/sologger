//! Lets several transports share the single global `log` logger.
//!
//! The `log` crate allows exactly one logger per process, so a binary with both the
//! logstash and OpenTelemetry transports cannot install both. Build each transport's
//! logger without installing it, wrap them in a [`FanoutLogger`], and install that once.

use log::{LevelFilter, Log, Metadata, Record};

/// Forwards every record to each logger that accepts it.
pub struct FanoutLogger {
    loggers: Vec<Box<dyn Log>>,
}

impl FanoutLogger {
    pub fn new(loggers: Vec<Box<dyn Log>>) -> Self {
        Self { loggers }
    }
}

impl Log for FanoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(metadata))
    }

    fn log(&self, record: &Record) {
        for logger in &self.loggers {
            if logger.enabled(record.metadata()) {
                logger.log(record);
            }
        }
    }

    fn flush(&self) {
        for logger in &self.loggers {
            logger.flush();
        }
    }
}

/// Applies a level to one logger, so each transport inside a [`FanoutLogger`] keeps its
/// own verbosity instead of sharing the process-wide maximum.
pub struct LevelFiltered<L> {
    inner: L,
    level: LevelFilter,
}

impl<L: Log> LevelFiltered<L> {
    pub fn new(inner: L, level: LevelFilter) -> Self {
        Self { inner, level }
    }

    pub fn level(&self) -> LevelFilter {
        self.level
    }
}

impl<L: Log> Log for LevelFiltered<L> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level && self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.inner.log(record);
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}
