//!
//! # 感知层服务
//! 感知层服务对接系统感知层设备，为感知层设备提供稳定可靠TCP连接、redis数据转发
//!

pub mod connection;
pub mod device;
pub mod map2redis;