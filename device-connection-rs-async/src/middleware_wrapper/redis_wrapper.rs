#[allow(unused_imports)]
use log::{
    error,
    info,
};
use redis::{
    aio::Connection,
    AsyncCommands,
    Client,
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
    pub async fn push_to_list(&mut self, list_name: &str, v: &str) -> Result<(), ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        match cli.lpush::<&str, &str, usize>(list_name, v).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("push msg({}) to list({}) failed: {:?}", v, list_name, e);
                Err(())
            }
        }
    }


    pub async fn pop_from_list(&mut self, list_name: &str) -> Result<String, ()> {
        let cli = if let Some(conn) = &mut self.conn {
            conn
        } else {
            error!("Redis conn in RedisConn struct is None");
            return Err(());
        };

        match cli.rpop::<&str, String>(list_name).await {
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

    if let Ok(keys) = keys_query {
        println!("keys query {:?}", keys.len());
    }

    redis::cmd("SET")
        .arg(&["key2", "bar"])
        .query_async(&mut con)
        .await?;

    let result: RedisResult<(String, Vec<u8>)> = redis::cmd("MGET")
        .arg(&["key1", "key2"])
        .query_async(&mut con)
        .await;

    println!("{:?}", result);
    Ok(())
}