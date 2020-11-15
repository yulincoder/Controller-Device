#[allow(unused_imports)]
use log::{error, info, warn};
use tokio::time::Duration;
use tokio::time::delay_for;

use super::redis_wrapper as rw;

const REDIS_ADDR: &str = "172.20.88.128";
const REDIS_PORT: &str = "6379";

pub const NAMESPACE_DEVICES_BORN: &str = "csod/devices_born";
pub const NAMESPACE_DEVICES_ALIVE: &str = "csod/devices_alive";
pub const NAMESPACE_DEVICE_STATUS: &str = "csod/device_status";
//pub const NAMESPACE_DEVICES_COMMON_EVENT_NOTIFY: &str = "csod/mq/p5";

pub async fn get_devices_num() -> Result<Option<String>, String> {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        warn!("connection redis fail.");
        return Err("connection redis fail.".to_string());
    };

    let rv = redis_conn.zcard(NAMESPACE_DEVICES_BORN).await;
    if let Ok(Some(v)) = rv {
        Ok(Some(format!("{}", v)))
    } else {
        warn!("read failed");
        Err("read fail.".to_string())
    }
}


pub async fn get_alive_devices_num() -> Result<Option<String>, String> {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        warn!("connection redis fail.");
        return Err("connection redis fail.".to_string());
    };

    let rv = redis_conn.zcard(NAMESPACE_DEVICES_ALIVE).await;
    if let Ok(Some(v)) = rv {
        Ok(Some(format!("{}", v)))
    } else {
        warn!("read fail.");
        Err("read fail.".to_string())
    }
}

pub async fn sn_is_alive(sn: &str) -> bool {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        return false;
    };

    let rv = redis_conn.zrank(NAMESPACE_DEVICES_ALIVE, sn).await;
    if let Ok(Some(_)) = rv {
        true
    } else {
        false
    }
}


// 写下行消息
async fn write_downlink(sn: &str, msg: &str) -> Result<Option<usize>, String> {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        warn!("dev {} {}", sn, "connection redis fail.".to_string());
        return Err("connection redis fail.".to_string());
    };
    info!("push msg to downlink {}", msg);
    match redis_conn.hset(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, sn), "downlink", msg).await {
        Err(_) => {
            warn!("dev {} {}", sn, "set redis fail.".to_string());
            Err("set redis fail.".to_string())
        }
        Ok(v) => {
            Ok(v)
        }
    }
}

// 清除上行ack
pub async fn clear_uplink(sn: &str) -> Result<(), String> {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        warn!("dev {} {}", sn, "connection redis fail.".to_string());
        return Err("connection redis fail.".to_string());
    };

    if let Ok(_) = redis_conn.hdel(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, sn), "uplink").await {
        Ok(())
    } else {
        warn!("dev {} {}", sn, "clear redis hash uplink msg failed".to_string());
        Err("clear redis hash uplink msg failed".to_string())
    }
}

// 读上行ack消息
pub async fn readline_uplink(sn: &str) -> Result<String, String> {
    let mut redis_conn = if let Ok(e) = rw::RedisConn::new(REDIS_ADDR, REDIS_PORT).await {
        e
    } else {
        warn!("dev {} {}", sn, "connection redis fail.".to_string());
        return Err("connection redis fail.".to_string());
    };

    if let Ok(v) = redis_conn.hget(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, sn), "uplink").await {
        // 读完值就从redis删除掉
        // TODO: 这里删除失败没处理
        let _ = redis_conn.hdel(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, sn), "uplink").await;
        Ok(v)
    } else {
        warn!("dev {} {}", sn, "read redis hash fail".to_string());
        Err("read redis hash fail".to_string())
    }
}


pub async fn transparent_transmit_wit_ack(sn: &str, msg: &str) -> Result<Option<String>, String> {
    // 先清除ack
    if let Err(e) = clear_uplink(&sn).await {
        warn!("dev {} clear ack failed", sn);
        return Err(e);
    }

    if let Err(e) = write_downlink(sn, msg).await {
        warn!("dev {} write downlink msg failed", sn);
        return Err(e);
    }

    // 等待ack最多5s
    for i in 1..50 {
        if let Ok(rv) = readline_uplink(sn).await {
            return Ok(Some(format!("{}", rv)));
        } else {
            warn!("dev {}, delay {}", sn, i);
            delay_for(Duration::from_millis(100)).await;
        }
    };
    Ok(None)
}