use anyhow::Result as AnyResult;
use log::{debug, error};
use log4rs::init_file;
use qoollo_log4rs_logstash::config::DeserializersExt;

/// Initialize the logger with the provided logstash config location
#[cfg(feature = "logstash")]
pub fn init_logstash_logger(log4rs_config_location: &String) -> AnyResult<()> {
    match {
        init_file(
            log4rs_config_location,
            log4rs::config::Deserializers::default().with_logstash(),
        )
    } {
        Ok(_) => {
            debug!("Logger initialized with logstash successfully")
        }
        Err(err) => {
            error!("init_logstash_logger not initialized! {}", err.to_string())
        }
    };
    Ok(())
}
