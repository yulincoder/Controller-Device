#![feature(test)]

mod application_service;
mod perception_service;
mod middleware_wrapper;
mod common;

use perception_service::connection as device_conn;

use common::config;

fn main() -> Result<(), ()>{
    // 加载所有配置
    let config: config::Config  = config::load_config("cfg.toml", true);
    config::log_init(config.log);

    device_conn::start(config.perception_service)?;
    Ok(())
}
