#![cfg(feature = "logstash")]

use crate::a::foo;
use sologger_log_transport::logstash_lib::init_logstash_logger;

#[test]
pub fn test_load_config() {
    let result = init_logstash_logger(&"./tests/configs/log4rs-config.json".to_string());
    foo();
    assert!(result.is_ok())
}

mod a {
    use log::error;

    pub fn foo() {
        error!("bar");
    }
}
