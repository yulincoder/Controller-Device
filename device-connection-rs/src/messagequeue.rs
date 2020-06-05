//! A Redis based Message Queue
//!
//! ```
//! let mut mq = if let Ok(mq) = messagequeue::MQ::new("redis://127.0.0.1") {
//!     mq
//! } else {
//!     panic!("MQ Create Failed")
//! };
//! for i in 1..2001 {
//!     msg.push_str(format!("{}", i).as_str());
//!     mq.push(&msg);
//! }
//! for i in 1..2000 {
//!     info!("------------ {}", i);
//!     if let Ok(msg) = mq.bpop(5) {
//!     info!("MQ {:?}", msg);
//! } else {
//!     warn!("MQ {}", "null data");
//! }
//! ```
#![allow(dead_code)]
use log::{error, info, warn};
use redis::Commands;

pub struct MQ {
    conn: redis::Connection,
    priority: String,
}

/// Creat connection to redis
/// ```
/// let mut mq = if let Ok(mq) = messagequeue::MQ::new("redis://127.0.0.1") {
///     mq
/// } else {
///     panic!("MQ Create Failed")
/// };
/// ```
fn redis_connection(addr: &'static str) -> Result<redis::Connection, ()> {
    //let addr = "redis://127.0.0.1";
    match redis::Client::open(addr) {
        Ok(client) => match client.get_connection() {
            Ok(con) => {
                info!("redis connect succeed");
                Ok(con)
            }
            Err(_) => {
                error!("redis connect fail, to check the redis-server whether be launch");
                Err(())
            }
        },
        Err(_) => {
            error!("redis connect fail, to check the redis-server whether be launch");
            Err(())
        }
    }
}

impl MQ {
    pub fn new(addr: &'static str) -> Result<MQ, ()> {
        match self::redis_connection(addr) {
            Ok(conn) => Ok(MQ {
                conn: conn,
                priority: String::from("p5"),
            }),
            _ => Err(()),
        }
    }

    /// Set the priority of the redis mq. default was "p5"
    pub fn set_priority(&mut self, p: String) {
        self.priority = p;
    }

    /// Push a message to the queue.
    pub fn push(&mut self, msg: &String) -> Result<(), ()> {
        match self
            .conn
            .lpush::<&String, &String, usize>(&self.priority, msg)
        {
            Ok(_) => {
                info!("push a msg({}) succeed", msg);
                Ok(())
            }
            _ => {
                info!("push a msg({}) succeed", msg);
                Err(())
            }
        }
    }

    /// Pop a message from the queue with blocking
    pub fn bpop(&mut self, timeout: usize) -> Result<String, ()> {
        match self
            .conn
            .brpop::<&String, Vec<String>>(&self.priority, timeout)
        {
            Ok(o) => {
                if o.len() > 0 {
                    info!("pop a message: {:?}, vec message len: {}", o, o.len());
                    Ok(o[1].clone())
                } else {
                    warn!("pop a message timeout: {:?}", o);
                    Err(())
                }
            }
            Err(e) => {
                warn!("pop message fial: {}", e);
                Err(())
            }
        }
    }

    /// Pop a message from the queue with no blocking.
    pub fn pop(&mut self) -> Result<String, ()> {
        match self.conn.rpop::<&String, String>(&self.priority) {
            Ok(o) => {
                if o.len() > 0 {
                    info!("pop a message: {:?}", o);
                    Ok(o.clone())
                } else {
                    info!("pop a message null: {:?}", o);
                    Err(())
                }
            }
            Err(e) => {
                info!("pop message null: {}", e);
                Err(())
            }
        }
    }

    /// Pop a message from the queue with no blocking.
    pub fn pop_sn(&mut self, sn: &String) -> Result<String, ()> {
        match self.conn.rpop::<&String, String>(sn) {
            Ok(o) => {
                if o.len() > 0 {
                    info!("pop a message: {:?}", o);
                    Ok(o.clone())
                } else {
                    Err(())
                }
            }
            Err(_) => Err(()),
        }
    }
}
