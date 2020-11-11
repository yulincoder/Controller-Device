#[allow(unused_imports)]
use log::{
    error,
    info,
    warn,
};

use crate::middleware_wrapper::redis_wrapper::{
    NAMESPACE_DEVICE_STATUS,
    NAMESPACE_DEVICES_BORN,
    NAMESPACE_DEVICES_COMMON_EVENT_NOTIFY,
};

use super::device::Device;
use super::super::middleware_wrapper::redis_wrapper::RedisConn;

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

    /// 更新设备在线状态，用于每次收到消息，就更新设备在线, 主要更新status online字段和添加online
    pub async fn update_online_status(&mut self) -> bool {
        info!("update 'online' failed: sn {}",self.dev.sn);
        if self.redis_conn.hset(&format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "online", "true").await.is_err() {
            error!("update 'online' failed: sn {}",self.dev.sn);
            return false
        }

        if let Err(_) = self.redis_conn.zadd_device_alive_with_timestamp(&self.dev.sn).await {
            error!("set redis fail");
            return false;
        }
        true
    }

    pub async fn activate(&mut self) -> bool {

        // 添加到在线设备集合中
        if let Err(_) = self.redis_conn.zadd_device_alive_with_timestamp(&self.dev.sn).await {
            error!("set redis fail");
            return false;
        }

        // 设置status在线状态
        if let Err(_) = self.redis_conn.hset_online_with_time(&format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "true").await {
            let _ = self.redis_conn.zrem_devices_alive(&self.dev.sn).await;
            error!("set redis fail");
            return false;
        }

        // 清理上行链路和下行链路缓存
        let _ = self.redis_conn.hdel(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "uplink").await;
        let _ = self.redis_conn.hdel(&*format!("{}{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "downlink").await;

        // 如果第一次激活的设备，添加到born有序集合中
        match self.redis_conn.zrank(NAMESPACE_DEVICES_BORN, &self.dev.sn).await {
            Ok(e) => {
                if let None = e {
                    if let Err(_) = self.redis_conn.zadd_device_born_with_timestamp(&self.dev.sn).await {
                        warn!("add the born set fail");
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            Err(_) => {
                false
            }
        }
    }

    pub async fn deactivate(&mut self) -> bool {
        warn!("device offline");
        // 设置在线状态为false
        let v1 = match self.redis_conn.hset_online_with_time(&format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "false").await {
            Ok(_) => true,
            Err(_) => {
                error!("set redis fail");
                false
            }
        };

        // 从在线设备集合中删除
        let v2 = match self.redis_conn.zrem_devices_alive(&self.dev.sn).await {
            Ok(_) => true,
            Err(_) => {
                error!("set redis fail");
                false
            }
        };
        v1 && v2
    }

    pub async fn readline_downlink(&mut self) -> Result<String, ()> {
        if let Ok(v) = self.redis_conn.hget(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "downlink").await {
            // 读完值就从redis删除掉
            let _ = self.redis_conn.hdel(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "downlink").await;
            Ok(v)
        } else {
            Err(())
        }
    }

    // 写上行ack消息
    pub async fn write_uplink(&mut self, msg: &str) -> Result<Option<usize>, ()> {
        info!("push msg to uplink {}", msg);
        self.redis_conn.hset(&*format!("{}/{}", NAMESPACE_DEVICE_STATUS, self.dev.sn), "uplink", msg).await
    }

    /// 所有设备的event消息都推入相同的key为"csod/mq/p5"队列
    pub async fn notify_event(&mut self, msg: &str) -> Result<Option<usize>, ()> {
        self.redis_conn.push_to_list(NAMESPACE_DEVICES_COMMON_EVENT_NOTIFY, msg).await
    }
}

/// redis基本测试
#[cfg(test)]
mod test_redis_conn {
    use futures::executor::block_on;

    use super::Device;
    use super::Device2redis;

    #[test]
    fn test_deactivate() {
        let mut d2r = block_on(Device2redis::new(Device::new("test".to_string()), "127.0.0.1", "6379"));
        //block_on(d2r.unwrap().activate());
        //assert_eq!(Ok(_), block_on(d2r.unwrap().activate()));
    }
}