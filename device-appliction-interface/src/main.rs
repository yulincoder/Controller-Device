#[allow(unused_imports)]
use log::{
    error,
    info,
};

use common::config;
use http_service as hs;

mod http_service;
mod middleware;
mod common;

fn main() {

    // 加载配置
    let sysconf: config::Config = config::load_config("cfg.toml", true);
    config::log_init(sysconf.log);

    hs::main::launch(sysconf.http_service, sysconf.redis).expect("error inside occurs");
}