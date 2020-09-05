//!
//! 基于Redis lpush/rpop操作的MQ
//!
use log::error;

use super::redis_wrapper::RedisConn;

pub struct MQ {
    redis_conn: RedisConn,
    default_priority: String, // 默认优先级
}

#[allow(dead_code)]
impl MQ {
    /// 获取一个MQ的连接，当前是redis连接
    pub async fn new(ip: &str, port: &str) -> Result<MQ, ()> {
        match RedisConn::new(ip, port).await {
            Ok(conn) => Ok(MQ { redis_conn: conn, default_priority: String::from("p5") }),
            Err(e) => {
                error!("cannot create connection to redis: {:?}", e);
                Err(())
            }
        }
    }

    /// 推入一条消息到默认优先级(p5)
    pub async fn push(&mut self, msg: &str) -> Result<(), ()> {
        self.redis_conn.push_to_list(&self.default_priority, msg).await
    }

    /// 推入一条消息到指定优先级队列
    pub async fn push_p(&mut self, priority: &str, msg: &str) -> Result<(), ()> {
        self.redis_conn.push_to_list(priority, msg).await
    }

    /// 从默认优先级(p5)获取一条消息
    pub async fn pop(&mut self) -> Result<String, ()> {
        self.redis_conn.pop_from_list(&self.default_priority).await
    }

    /// 从指定优先级获取一条消息
    pub async fn pop_p(&mut self, priority: &str) -> Result<String, ()> {
        self.redis_conn.pop_from_list(priority).await
    }

    /// 清空默认优先级队列
    pub async fn clear(&mut self) {
        self.redis_conn.del_key(&self.default_priority).await;
    }

    /// 清空指定优先级队列
    pub async fn clear_p(&mut self, priority: &str) {
        self.redis_conn.del_key(priority).await;
    }
}

