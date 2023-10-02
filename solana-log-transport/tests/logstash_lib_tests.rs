use solana_log_transport::logstash_lib::init_logstash_logger;
use crate::a::foo;

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
