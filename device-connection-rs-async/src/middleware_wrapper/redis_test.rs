extern crate test;

#[allow(unused_imports)]
use test::Bencher;

#[allow(unused_imports)]
use futures::{
    executor::block_on,
};
#[allow(unused_imports)]
use redis::{
    aio::Connection,
    AsyncCommands,
    Client,
    RedisResult,
};

/// redis基本测试
#[cfg(test)]
mod test_redis_conn {
    use std::borrow::Borrow;

    use futures::executor::block_on;

    use crate::config;
    use crate::middleware_wrapper::redis_wrapper::{NAMESPACE_DEVICES_BORN, RedisConn};

    use super::Bencher;

    /// 测试hash
    #[test]
    fn test_hash() {
        let config: config::Config = config::load_config("cfg.toml", true);
        let redis_conn = block_on(RedisConn::new(&config.redis.ip.unwrap(), &config.redis.port.unwrap()));
        let mut conn = if let Ok(instance) = redis_conn {
            instance
        } else {
            assert!(false);
            panic!("redis connection false");
        };

        // 设置一个值
        let sn = "sn_test_1234";
        let _ = block_on(conn.del_key("sn_test_1234"));
        let _ = block_on(conn.hset(sn, "online", "true"));
        let mut status = block_on(conn.hget(sn, "online"));
        assert_eq!("true".to_string(), status.unwrap());

        // 删除
        let _ = block_on(conn.hdel(sn, "status"));
        status = block_on(conn.hget(sn, "status"));
        assert_eq!(Err(()), status);

        let _ = block_on(conn.hset_online_with_time(sn, "true"));
        status = block_on(conn.hget(sn, "online"));
        assert_eq!("true".to_string(), status.unwrap());
    }

    /// 测试有序集合recent key插入、计数
    #[test]
    fn test_sorted_set() {
        let config: config::Config = config::load_config("cfg.toml", true);
        let redis_conn = block_on(RedisConn::new(&config.redis.ip.unwrap(), &config.redis.port.unwrap()));
        let mut conn = if let Ok(instance) = redis_conn {
            instance
        } else {
            assert!(false);
            panic!("redis connection false");
        };

        block_on(conn.del_key(NAMESPACE_DEVICES_BORN));

        for i in 0..100 {
            if block_on(conn.zadd_device_born_with_timestamp(&format!("sn_test_{}", i))).is_err() {
                assert!(false);
            }
        }

        assert_eq!(Ok(100), block_on(conn.zcard_devices_born()));
    }

    /// 测试redis连接->set值->get值
    #[test]
    fn test_set_get() {
        let config: config::Config = config::load_config("cfg.toml", true);
        let redis_conn = block_on(RedisConn::new(&config.redis.ip.unwrap(), &config.redis.port.unwrap()));
        let mut conn = if let Ok(instance) = redis_conn {
            instance
        } else {
            assert!(false);
            panic!("redis connection false");
        };

        if block_on(conn.set("test__", "test_fuck__")).is_err() {
            assert!(false);
        };

        match block_on(conn.get("test__")) {
            Ok(v) => {
                assert_eq!(v, String::from("test_fuck__"));
            }
            Err(_) => {
                assert!(false);
            }
        }
    }

    /// 测试redis队列push->pop
    #[test]
    fn test_push_pop() {
        let config: config::Config = config::load_config("cfg.toml", true);
        let redis_conn = block_on(RedisConn::new(&config.redis.ip.unwrap(), &config.redis.port.unwrap()));
        let mut conn = if let Ok(instance) = redis_conn {
            instance
        } else {
            assert!(false);
            panic!("redis connection false");
        };

        block_on(conn.del_key("test-p5"));

        for i in 0..=100 {
            if block_on(conn.push_to_list("test-p5", &format!("fuck-{}", i))).is_err() {
                assert!(false);
            }
        }

        for i in 0..=100 {
            if let Ok(v) = block_on(conn.pop_from_list("test-p5")) {
                assert_eq!(format!("fuck-{}", i), v);
            } else {
                assert!(false);
            }
        }

        assert!(true);
    }

    #[bench]
    fn bench_redis_conn_new(b: &mut Bencher) {
        let config: config::Config = config::load_config("cfg.toml", true);
        b.iter(|| {
            let redis_conn = block_on(RedisConn::new(&config.redis.ip.borrow().as_ref().unwrap(), &config.redis.port.borrow().as_ref().unwrap()));
            if redis_conn.is_err() {
                assert!(false);
            };
        });
    }
}

#[cfg(test)]
mod test_mq {
    use std::borrow::Borrow;

    use futures::executor::block_on;

    use crate::config;

    use super::Bencher;
    use super::super::mq::MQ;

    /// 测试队列push->pop
    #[test]
    fn test_push_pop() {
        let config: config::Config = config::load_config("cfg.toml", true);
        let mq_conn = block_on(MQ::new(&config.redis.ip.unwrap(), &config.redis.port.unwrap()));
        let mut mq = if let Ok(instance) = mq_conn {
            instance
        } else {
            assert!(false);
            panic!("redis connection false");
        };

        block_on(mq.clear());

        for i in 0..=100 {
            if block_on(mq.push(&format!("fuck-{}", i))).is_err() {
                assert!(false);
            }
        }
        for i in 0..=100 {
            if let Ok(v) = block_on(mq.pop()) {
                assert_eq!(format!("fuck-{}", i), v);
            }
        }
    }

    /// 基准测试队列push
    #[bench]
    fn bench_mq_conn_new(b: &mut Bencher) {
        let config: config::Config = config::load_config("cfg.toml", true);
        b.iter(|| {
            let mq_conn = block_on(MQ::new(&config.redis.ip.borrow().as_ref().unwrap(), &config.redis.port.borrow().as_ref().unwrap()));
            if mq_conn.is_err() {
                assert!(false);
            };
        });
    }
}

#[allow(dead_code)]
async fn redis_set_key(con: &mut Connection) -> redis::RedisResult<()> {
    con.set(format!("fukkkkkkkkk : {}", 12), b"fucko").await?;
    Ok(())
}

#[allow(dead_code)]
async fn redis_conn() -> redis::RedisResult<()> {
    let client: Client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut _con: Connection = client.get_async_connection().await?;
    Ok(())
}

#[bench]
fn bench_redis_conn(b: &mut Bencher) {
    b.iter(|| block_on(redis_conn()));
}

#[bench]
fn bench_redis_set_key(b: &mut Bencher) {
    let client: Client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con: Connection = block_on(client.get_async_connection()).unwrap();
    b.iter(|| block_on(redis_set_key(&mut con)));
}

/*
``` redis-rs async example:
pub async fn test() -> redis::RedisResult<()> {
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
```
*/