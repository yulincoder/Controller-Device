#[allow(unused_imports)]
use log::{
    error,
    info,
};

use super::device::Device;
use super::super::middleware_wrapper::redis_wrapper::RedisConn;

static g_status_suffix: &str = "_status";
static g_uplink_msg_suffix: &str = "_uplink";
static g_downlink_msg_suffix: &str = "_downlinkmsg";
static g_conn_id_suffix: &str = "_connid";


pub struct Device2redis {
    pub dev: Device,
    pub redis_conn: RedisConn,
}

impl Device2redis {
    pub async fn new(dev: Device, ip: &str, port: &str) -> Result<Device2redis, ()> {
        if let Ok(conn) = RedisConn::new(ip, port).await {
            Ok(Device2redis { dev: dev, redis_conn: conn })
        } else {
            error!("connection redis fail, ");
            Err(())
        }
    }

    pub async fn activate(&mut self) -> bool {
        if let Ok(_) = self.redis_conn.set(&*format!("{}{}", self.dev.sn, g_status_suffix), "online").await {
            self.redis_conn.del_key(&*format!("{}{}", self.dev.sn, g_uplink_msg_suffix)).await;
            self.redis_conn.del_key(&*format!("{}{}", self.dev.sn, g_downlink_msg_suffix)).await;
            true
        } else {
            error!("set redis fail");
            false
        }
    }

    pub async fn deactivate(&mut self) -> bool {
        if let Ok(_) = self.redis_conn.set(&*format!("{}{}", self.dev.sn, g_status_suffix), "offline").await {
            self.redis_conn.del_key(&*format!("{}{}", self.dev.sn, g_uplink_msg_suffix)).await;
            self.redis_conn.del_key(&*format!("{}{}", self.dev.sn, g_downlink_msg_suffix)).await;
            true
        } else {
            error!("set redis fail");
            false
        }
    }

    pub async fn readline_downlink(&mut self) -> Result<String, ()> {
        if let Ok(v) = self.redis_conn.get(&*format!("{}{}", self.dev.sn, g_downlink_msg_suffix)).await {
            // 读完值就从redis删除掉
            self.redis_conn.del_key(&*format!("{}{}", self.dev.sn, g_downlink_msg_suffix)).await;
            Ok(v)
        } else {
            Err(())
        }
    }
}