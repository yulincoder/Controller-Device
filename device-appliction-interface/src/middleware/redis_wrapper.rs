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

#[allow(dead_code)]
impl RedisConn {
    /// 建立一个redis async连接
    pub async fn new(ip: &str, port: &str) -> RedisResult<RedisConn> {
        let _redis_addr = format!("redis://{}:{}/", ip, port);
        let client_result = Client::open(_redis_addr);

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

    pub async fn zcard(&mut self, key: &str) -> Result<Option<u64>, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        let query: RedisResult<Option<u64>> = redis_cmd("zcard")
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
}