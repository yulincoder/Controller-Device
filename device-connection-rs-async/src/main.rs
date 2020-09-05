#![feature(test)]

use std::thread;

#[allow(unused_imports)]
use log::{
    error,
    info,
};

use common::config;
use perception_service::connection as device_conn;

mod application_service;
mod perception_service;
mod middleware_wrapper;
mod common;

fn main() -> Result<(), ()> {

    // 加载配置
    let config: config::Config = config::load_config("cfg.toml", true);
    config::log_init(config.log);

    // 新建Thread, 传入配置，启动感知层连接服务
    let cfg_perception_service = config.perception_service;
    let cfg_redis = config.redis;
    let perception_service_work = thread::spawn(move || {
        device_conn::start(cfg_perception_service, cfg_redis).unwrap();
    });

    if let Err(_) = perception_service_work.join() {
        panic!("start perception service failed")
    }

    Ok(())
}
