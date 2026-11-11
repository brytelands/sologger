//! `FanoutLogger` lets several transports share the single global `log` logger, and
//! `LevelFiltered` gives each its own level.

use std::sync::{Arc, Mutex};

use log::{Level, LevelFilter, Log, Metadata, Record};
use sologger_log_transport::fanout::{FanoutLogger, LevelFiltered};

/// Records every message it is asked to log.
#[derive(Clone, Default)]
struct Capture {
    messages: Arc<Mutex<Vec<String>>>,
    flushes: Arc<Mutex<usize>>,
}

impl Capture {
    fn messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

impl Log for Capture {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        self.messages
            .lock()
            .unwrap()
            .push(format!("{} {}", record.level(), record.args()));
    }
    fn flush(&self) {
        *self.flushes.lock().unwrap() += 1;
    }
}

fn emit(logger: &dyn Log, level: Level, message: &str) {
    logger.log(
        &Record::builder()
            .level(level)
            .target("sologger::test")
            .args(format_args!("{}", message))
            .build(),
    );
}

fn metadata(level: Level) -> Metadata<'static> {
    Metadata::builder()
        .level(level)
        .target("sologger::test")
        .build()
}

#[test]
fn every_logger_receives_each_record() {
    let (a, b) = (Capture::default(), Capture::default());
    let fanout = FanoutLogger::new(vec![Box::new(a.clone()), Box::new(b.clone())]);

    emit(&fanout, Level::Info, "tx parsed");

    assert_eq!(a.messages(), vec!["INFO tx parsed"]);
    assert_eq!(b.messages(), vec!["INFO tx parsed"]);
}

#[test]
fn each_logger_keeps_its_own_level() {
    let (verbose, quiet) = (Capture::default(), Capture::default());
    let fanout = FanoutLogger::new(vec![
        Box::new(LevelFiltered::new(verbose.clone(), LevelFilter::Info)),
        Box::new(LevelFiltered::new(quiet.clone(), LevelFilter::Error)),
    ]);

    emit(&fanout, Level::Info, "routine");
    emit(&fanout, Level::Error, "failed tx");

    assert_eq!(verbose.messages(), vec!["INFO routine", "ERROR failed tx"]);
    assert_eq!(quiet.messages(), vec!["ERROR failed tx"]);
}

#[test]
fn enabled_when_any_logger_wants_the_level() {
    let fanout = FanoutLogger::new(vec![
        Box::new(LevelFiltered::new(Capture::default(), LevelFilter::Warn)),
        Box::new(LevelFiltered::new(Capture::default(), LevelFilter::Error)),
    ]);

    assert!(fanout.enabled(&metadata(Level::Warn)));
    assert!(!fanout.enabled(&metadata(Level::Info)));
}

#[test]
fn flush_reaches_every_logger() {
    let (a, b) = (Capture::default(), Capture::default());
    let fanout = FanoutLogger::new(vec![Box::new(a.clone()), Box::new(b.clone())]);

    fanout.flush();

    assert_eq!(*a.flushes.lock().unwrap(), 1);
    assert_eq!(*b.flushes.lock().unwrap(), 1);
}

#[test]
fn level_filtered_reports_its_level() {
    assert_eq!(
        LevelFiltered::new(Capture::default(), LevelFilter::Warn).level(),
        LevelFilter::Warn
    );
}
