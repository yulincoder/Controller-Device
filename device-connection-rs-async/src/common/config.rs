//!
//! # 加载配置项
//!
//! Example
//! ```
//! use common::config;
//!
//! fn main() -> Result<(), ()>{
//!     // 加载cfg.toml内所有配置
//!     let config: config::Config  = config::load_config("cfg.toml", true);
//!     println!("{:?}", config.log); // 打印log配置
//!     println!("{:?}", config.perception_service); // 打印感知层配置
//!     println!("{:?}", config.application_service); // 打印应用层配置
//!     Ok(())
//! }
//! ```
//!

use std::env;
use std::fs::File;
use std::io::Read;

use loge;
use serde_derive::Deserialize;
use toml;

#[derive(Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct LogConfig {
    pub level: Option<String>,
    pub outfile: Option<String>,
    pub format: Option<String>,
}

#[derive(Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct PerceptionServiceConfig {
    pub ip: Option<String>,
    pub port: Option<String>,
    pub heartbeat_interval: Option<u64>,
}

#[derive(Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct RedisConfig {
    pub ip: Option<String>,
    pub port: Option<String>,
}

#[derive(Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Config {
    pub log: LogConfig,
    pub perception_service: PerceptionServiceConfig,
    pub redis: RedisConfig,
}

/// Init to the logger
#[allow(dead_code)]
pub fn log_init(log_cfg: LogConfig) {
    let level: Option<String> = log_cfg.level;
    let outfile: Option<String> = log_cfg.outfile;
    let format: Option<String> = log_cfg.format;
    match &level {
        Some(le) if le == &"none".to_string() => {
            return;
        }
        _ => {
            if let Some(le) = level {
                env::set_var("RUST_LOG", le);
            } else {
                env::set_var("RUST_LOG", "trace");
            }
            if let Some(fmt) = format {
                env::set_var("LOGE_FORMAT", fmt);
            } else {
                env::set_var("LOGE_FORMAT", "fileline");
            }
            if let Some(f) = outfile {
                loge::init_with_file(f);
            }
        }
    }
}

#[allow(dead_code)]
pub fn load_config(toml_path: &str, verbose: bool) -> Config {
    let mut file = File::open(toml_path).unwrap();
    let mut file_content = String::new();
    if let Err(_) = file.read_to_string(&mut file_content) {
        panic!("read config error")
    }
    let config: Config = match toml::from_str(&file_content) {
        Ok(c) => c,
        _ => {
            panic!("parse config toml error");
        }
    };

    if verbose {
        println!("============== loading config =============");
        println!("[log]: \n\tlevel = {:?}\n\toutfile = {:?}\n\tformat = {:?}", if let Some(e) = &config.log.level {
            e.clone()
        } else {
            "Null".to_string()
        }, if let Some(e) = &config.log.outfile {
            e.clone()
        } else {
            "Null".to_string()
        }, if let Some(e) = &config.log.format {
            e.clone()
        } else {
            "Null".to_string()
        });

        println!("[perception connection]: \n\tip = {:?}\n\tport = {:?}\n\theartbeat_interval = {:?}", if let Some(e) = &config.perception_service.ip {
            e.clone()
        } else {
            "Null".to_string()
        }, if let Some(e) = &config.perception_service.port {
            e.clone()
        } else {
            "Null".to_string()
        }, if let Some(e) = &config.perception_service.heartbeat_interval {
            format!("{}", &e)
        } else {
            "Null".to_string()
        });

        println!("[redis]: \n\tip = {:?}\n\tport = {:?}", if let Some(e) = &config.redis.ip {
            e.clone()
        } else {
            "Null".to_string()
        }, if let Some(e) = &config.redis.port {
            e.clone()
        } else {
            "Null".to_string()
        });
        println!("===========================================");
    }
    config
}