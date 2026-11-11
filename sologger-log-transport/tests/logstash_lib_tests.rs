#![cfg(feature = "logstash")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::LevelFilter;
use sologger_log_transport::logstash_lib::{build_logstash_logger, init_logstash_logger};

const CONFIG: &str = "./tests/configs/log4rs-config.yml";

#[test]
fn builds_a_logger_from_the_config_without_installing_it() {
    let logstash = build_logstash_logger(CONFIG).expect("valid config");

    assert_eq!(logstash.refresh_rate(), Some(Duration::from_secs(5)));
    // root is `error`, but the `test::a` logger raises the maximum to `info`
    assert_eq!(logstash.max_level(), LevelFilter::Info);
}

#[test]
fn a_missing_config_is_an_error() {
    let missing = "./tests/configs/does-not-exist.yml";
    assert!(build_logstash_logger(missing).is_err());
    assert!(init_logstash_logger(&missing.to_string()).is_err());
}

#[test]
fn an_invalid_config_is_an_error() {
    let dir = scratch_dir("invalid");
    let path = dir.join("log4rs.yml");
    std::fs::write(&path, "root: [this is not a log4rs config").unwrap();

    assert!(build_logstash_logger(path.to_str().unwrap()).is_err());
}

#[test]
fn reloads_a_changed_config_and_reports_it() {
    let dir = scratch_dir("reload");
    let path = dir.join("log4rs.yml");
    std::fs::write(&path, console_config("warn")).unwrap();

    let logstash = build_logstash_logger(path.to_str().unwrap()).unwrap();
    assert_eq!(logstash.max_level(), LevelFilter::Warn);

    let reloads = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&reloads);
    let handle = logstash.handle();
    logstash.watch_config(move || {
        counter.fetch_add(1, Ordering::SeqCst);
    });

    std::fs::write(&path, console_config("error")).unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    while handle.max_log_level() != LevelFilter::Error && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(handle.max_log_level(), LevelFilter::Error);
    assert!(
        reloads.load(Ordering::SeqCst) >= 1,
        "after_reload not called"
    );
}

fn console_config(level: &str) -> String {
    format!(
        "refresh_rate: 1 seconds\n\
         appenders:\n  console:\n    kind: console\n\
         root:\n  level: {level}\n  appenders:\n    - console\n"
    )
}

fn scratch_dir(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("sologger-logstash-{}-{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
