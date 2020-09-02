#![feature(test)]

use std::thread;

mod application_service;
mod perception_service;
mod middleware_wrapper;
mod common;

use perception_service::connection as device_conn;

use common::config;

fn main() -> Result<(), ()>{

    // 加载配置
    let config: config::Config  = config::load_config("cfg.toml", true);
    config::log_init(config.log);

    // 新建Thread, 传入配置，启动感知层连接服务
    let cfg_perception_service = config.perception_service;
    let perception_service_work = thread::spawn(move || {
        device_conn::start(cfg_perception_service).unwrap();
    });

    perception_service_work.join();

    Ok(())
}
