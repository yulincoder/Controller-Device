use std::time::SystemTime;

#[allow(unused_imports)]
use log::{
    error,
    info,
};
use redis::{
    aio::Connection,
    AsyncCommands,
    Client,
    cmd as redis_cmd,
    RedisResult,
};

pub struct RedisConn {
    conn: Option<Connection>,
}

pub const NAMESPACE_DEVICES_BORN: &str = "csod/devices_born";
pub const NAMESPACE_DEVICES_ALIVE: &str = "csod/devices_alive";
pub const NAMESPACE_DEVICE_STATUS: &str = "csod/device_status";
pub const NAMESPACE_DEVICES_COMMON_EVENT_NOTIFY: &str = "csod/mq/p5";

#[allow(dead_code)]
impl RedisConn {
    /// 建立一个redis async连接
    pub async fn new(ip: &str, port: &str) -> RedisResult<RedisConn> {
        let _redis_addr = format!("redis://{}:{}/", ip, port);
        let client_result = redis::Client::open(_redis_addr);

        match client_result {
            Ok(cli) => {
                match cli.get_async_connection().await {
                    Ok(conn) => Ok(RedisConn { conn: Some(conn) }),
                    Err(e) => {
                        error!("get async redis connection fail: {:?}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                error!("connect redis fail: {:?}", e);
                Err(e)
            }
        }
    }

    pub async fn set(&mut self, k: &str, v: &str) -> Result<(), ()> {
        match &mut self.conn {
            Some(cli) => {
                match cli.set::<&str, &str, String>(k, v).await {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        error!("Set key to redis failed.");
                        Err(())
                    }
                }
            }
            _ => {
                error!("Redis conn in RedisConn struct is None");
                Err(())
            }
        }
    }

    pub async fn get(&mut self, key: &str) -> Result<String, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        match cli.get::<&str, String>(key).await {
            Ok(v) => Ok(v),
            Err(_) => Err(())
        }
    }

    /// 将数据从左端(lpush)推入指定redis list
    pub async fn push_to_list(&mut self, list_name: &str, v: &str) -> Result<Option<usize>, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        match cli.lpush::<&str, &str, Option<usize>>(list_name, v).await {
            Ok(e) => {
                info!("push msg({}) to list({}) , result: {:?}", v, list_name, e);
                Ok(e)
            }
            Err(e) => {
                error!("push msg({}) to list({}) failed: {:?}", v, list_name, e);
                Err(())
            }
        }
    }


    pub async fn pop_from_list(&mut self, list_name: &str) -> Result<Option<String>, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        match cli.rpop::<&str, Option<String>>(list_name).await {
            Ok(v) => Ok(v),
            Err(e) => {
                error!("pop from list({}) failed: {:?}", list_name, e);
                Err(())
            }
        }
    }

    pub async fn del_key(&mut self, key: &str) {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return;
        };

        let _: RedisResult<u16> = redis::cmd("del")
            .arg(key)
            .query_async(cli)
            .await;
    }

    pub async fn zcard(&mut self, key: &str) -> Result<u64, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<u64> = redis_cmd("zcard")
            .arg(key)
            .query_async(cli)
            .await;

        if let Ok(num) = query {
            Ok(num)
        } else {
            error!("{:?}", query);
            Err(())
        }
    }

    pub async fn zadd(&mut self, key: &str, score: &str, member: &str) -> Result<u64, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<u64> = redis_cmd("zadd")
            .arg(key).arg(score).arg(member)
            .query_async(cli)
            .await;

        if let Ok(num) = query {
            Ok(num)
        } else {
            Err(())
        }
    }

    /// 返回指定成员在有序集合中的索引
    pub async fn zrank(&mut self, key: &str, member: &str) -> Result<Option<usize>, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<Option<usize>> = redis_cmd("zrank")
            .arg(key).arg(member)
            .query_async(cli)
            .await;

        if let Ok(idx) = query {
            Ok(idx)
        } else {
            Err(())
        }
    }


    /// 删除有序集合中指定值
    pub async fn zrem(&mut self, key: &str, member: &str) -> Result<Option<usize>, ()> {
        println!("debug ::: key {}, member: {}", key, member);
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<Option<usize>> = redis_cmd("zrem")
            .arg(key).arg(member)
            .query_async(cli)
            .await;

        if let Ok(num) = query {
            Ok(num)
        } else {
            Err(())
        }
    }

    /// count the number of members in the the `csod/devices_born` key
    pub async fn zcard_devices_born(&mut self) -> Result<u64, ()> {
        self.zcard(NAMESPACE_DEVICES_BORN).await
    }

    /// add a member to sorted set with `csod/devices_born` key
    pub async fn zadd_devices_born(&mut self, score: &str, member: &str) -> Result<u64, ()> {
        self.zadd(NAMESPACE_DEVICES_BORN, score, member).await
    }

    /// count the number of members in the the `csod/devices_alive` key
    pub async fn zcard_devices_alive(&mut self) -> Result<u64, ()> {
        self.zcard(NAMESPACE_DEVICES_ALIVE).await
    }

    /// add a member to sorted set with `csod/devices_alive` key
    pub async fn zadd_devices_alive(&mut self, score: &str, member: &str) -> Result<u64, ()> {
        self.zadd(NAMESPACE_DEVICES_ALIVE, score, member).await
    }

    /// remove a member to sorted set with `csod/devices_born` key
    pub async fn zrem_devices_alive(&mut self, member: &str) -> Result<Option<usize>, ()> {
        self.zrem(NAMESPACE_DEVICES_ALIVE, member).await
    }


    pub fn get_unix_timestamp(&self) -> f64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(stamp) => stamp.as_micros() as f64 / 1e6,
            Err(_) => {
                error!("error timestamp, Should never be here");
                0.0f64
            }
        }
    }

    /// 向redis有序集合插入一个sn，其中插入时间戳为该sn score，时间戳个位为s，小数点保留到微秒
    pub async fn zadd_device_born_with_timestamp(&mut self, sn: &str) -> Result<u64, ()> {
        let born_stamp = self.get_unix_timestamp();
        match self.hset(&format!("{}/{}", NAMESPACE_DEVICE_STATUS, sn), "borntime", &born_stamp.to_string()).await {
            Ok(_) => self.zadd_devices_born(&format!("{}", born_stamp), sn).await,
            Err(e) => {
                error!("insert sorted fail: {:?}", e);
                Err(e)
            }
        }
    }

    /// 向redis有序集合插入一个sn，其中插入时间戳为该sn score，时间戳个位为s，小数点保留到微秒
    pub async fn zadd_device_alive_with_timestamp(&mut self, sn: &str) -> Result<u64, ()> {
        self.zadd_devices_alive(&format!("{}", self.get_unix_timestamp()), sn).await
    }


    /// 设置指定key的hash表，并设置指定字段和指定值
    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> Result<Option<usize>, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<Option<usize>> = redis_cmd("hset")
            .arg(key).arg(field).arg(value)
            .query_async(cli)
            .await;
        info!(" hset test key {}, field {}, value {}, {:?}", key, field, value, query);
        if let Ok(num) = query {
            Ok(num)
        } else {
            error!("set hash fail: {}, {}, {}, query: {:?}", key, field, value, query);
            Err(())
        }
    }

    /// 获取hash表指定key的指定字段
    pub async fn hget(&mut self, key: &str, field: &str) -> Result<String, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<String> = redis_cmd("hget")
            .arg(key).arg(field)
            .query_async(cli)
            .await;

        if let Ok(string) = query {
            Ok(string)
        } else {
            Err(())
        }
    }

    /// 删除hash表指定key的指定字段
    pub async fn hdel(&mut self, key: &str, field: &str) -> Result<u64, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<u64> = redis_cmd("hdel")
            .arg(key).arg(field)
            .query_async(cli)
            .await;

        if let Ok(num) = query {
            Ok(num)
        } else {
            Err(())
        }
    }

    /// 设置指定key的`toggletime`字段，并同时设置当前时间戳
    pub async fn hset_toggletime(&mut self, key: &str) -> Result<Option<usize>, ()> {
        self.hset(key, "toggletime", &format!("{}", self.get_unix_timestamp())).await
    }

    /// 设置指定key的`status`字段，并同时设置当前时间戳
    pub async fn hset_online_with_time(&mut self, key: &str, online: &str) -> Result<u64, ()> {
        self.hset(key, "online", online).await.is_ok();
        self.hset_toggletime(key).await;
        Ok(0)
    }
}


/// ### 获取设备数量
/// 表示曾经连接过云端的设备数量
#[allow(dead_code)]
pub async fn get_devices_num() {
    //TODO redis历史上线设备数量查询
}

/// ### 当前在线设备数量
#[allow(dead_code)]
pub async fn get_alive_devices_num() {
    //TODO redis当前在线设备数量插叙
}

#[allow(dead_code)]
pub async fn test_redis_async() -> redis::RedisResult<()> {
    let client: Client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con: Connection = client.get_async_connection().await?;

    for i in 3..100 {
        con.set(format!("fukkkkkkkkk : {}", i), b"fucko").await?;
    }


    let keys_query: RedisResult<Vec<String>> = redis::cmd("keys")
        .arg("*")
        .query_async(&mut con)
        .await;

    if let Ok(keys) = keys_query {}

    redis::cmd("SET")
        .arg(&["key2", "bar"])
        .query_async(&mut con)
        .await?;

    let result: RedisResult<(String, Vec<u8>)> = redis::cmd("MGET")
        .arg(&["key1", "key2"])
        .query_async(&mut con)
        .await;

    Ok(())
}