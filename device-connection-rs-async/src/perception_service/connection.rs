use log::{error, info, warn};
use serde_json;
use serde_json::Value;
use tokio::{
    io::BufReader,
    io::BufWriter,
    net::tcp::ReadHalf,
    net::tcp::WriteHalf,
    net::TcpListener,
    net::TcpStream,
    prelude::*,
    runtime,
    stream::StreamExt,
    time::Duration,
    time::timeout,
};

use tcp_err::ServerError;

use crate::common::config::PerceptionServiceConfig as PerceptCfg;
use crate::common::config::RedisConfig as RedisCfg;
use crate::perception_service::map2redis;

use super::device;

mod tcp_err {
    #[derive(Debug)]
    pub enum ServerError {
        Broken,
    }
}

#[derive(Debug, PartialEq)]
pub enum MESTYPE {
    HEARTBEAT,
    RAWDATA,
    INVAILD,
}

/// If the msg is a heartbeat ping type, return the sn
///
/// # Examples
/// ```
/// let ok_str = r#"{"type": "ping","sn": "123"}"#;
/// assert_eq!(carte::is_heartbeat(&ok_str.to_string()), Some("123".to_string()));
/// let err_str = r#"{"type": "ping","what": "error"}"#;
/// assert_ne!(carte::is_heartbeat(&err_str.to_string()), Some("123".to_string()));
/// ```
fn is_heartbeat(msg: &String) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                let msg_type: Option<String> = if t.contains_key("type") {
                    match t.get("type").unwrap() {
                        Value::String(v) => Some(v.to_string()),
                        _ => None,
                    }
                } else {
                    None
                };

                match msg_type {
                    Some(s) if s == "ping" => {
                        if t.contains_key("sn") {
                            match t.get("sn").unwrap() {
                                Value::String(v) => Some(v.to_string()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    sn
}

#[test]
fn is_heartbeat_test() {
    let ok_str = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(is_heartbeat(&ok_str.to_string()), Some("123".to_string()));
    let err_str = r#"{"type": "ping","what": "error"}"#;
    assert_ne!(is_heartbeat(&err_str.to_string()), Some("123".to_string()));
}

/// Get sn from a valid json.
pub fn parse_sn(msg: &String) -> Option<String> {
    if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                if t.contains_key("sn") {
                    match t.get("sn") {
                        Some(Value::String(v)) => Some(v.to_string()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

#[test]
fn parse_sn_test() {
    let ok_ping = r#"{"type": "ping","sn": "1234"}"#;
    assert_eq!(parse_sn(&ok_ping.to_string()), Some("1234".to_string()));
    let ok_rawdata = r#"{"type": "rawdata","sn": "145623"}"#;
    assert_eq!(
        parse_sn(&ok_rawdata.to_string()),
        Some("145623".to_string())
    );
    let err0_sn = r#"{"type": "rawdata","snfuck": "145623"}"#;
    assert_eq!(parse_sn(&err0_sn.to_string()), None);
    let err1_sn = r#"{"what": "error"}"#;
    assert_eq!(parse_sn(&err1_sn.to_string()), None);
    let err2_sn = r#"{"what": "err"#;
    assert_eq!(parse_sn(&err2_sn.to_string()), None);
}

///
/// When the msg is a valid json and include the `type` field,
/// but not the heartbeat type, it is a raw data.
pub fn is_rawdata(msg: &String) -> bool {
    if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                if t.contains_key("type") {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

pub fn parse_msg_type(msg: &String) -> MESTYPE {
    if let Some(_) = is_heartbeat(msg) {
        MESTYPE::HEARTBEAT
    } else if is_rawdata(msg) {
        MESTYPE::RAWDATA
    } else {
        MESTYPE::INVAILD
    }
}

#[test]
fn parse_msg_type_test() {
    let ok_ping = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(parse_msg_type(&ok_ping.to_string()), MESTYPE::HEARTBEAT);
    let ok_rawdata = r#"{"type": "rawdata","sn": "123"}"#;
    assert_eq!(parse_msg_type(&ok_rawdata.to_string()), MESTYPE::RAWDATA);
    let err1_rawdata = r#"{"what": "error"}"#;
    assert_eq!(parse_msg_type(&err1_rawdata.to_string()), MESTYPE::INVAILD);
    let err2_rawdata = r#"{"what": "err"#;
    assert_eq!(parse_msg_type(&err2_rawdata.to_string()), MESTYPE::INVAILD);
}


async fn readline<'a>(stream: &'a mut BufReader<ReadHalf<'_>>) -> Result<String, ServerError> {
    let mut line = String::new();
    match stream.read_line(&mut line).await {
        Ok(n) => {
            if n > 0 {
                Ok(line)
            } else {
                Err(ServerError::Broken)
            }
        }
        Err(e) => {
            warn!("connection error: {}", e);
            Err(ServerError::Broken)
        }
    }
}

/// 握手成功返回sn
async fn handshake<'a>(reader: &'a mut BufReader<ReadHalf<'_>>) -> Result<String, ()> {
    let pinsn: String;
    for _ in 0..4 {
        if let Ok(msg) = readline(reader).await {
            if let Some(sn) = is_heartbeat(&msg) {
                info!("sn {}", sn);
                pinsn = sn.clone().trim().to_string();
                return Ok(pinsn);
            }
        }
        tokio::time::delay_for(Duration::from_millis(100)).await;
    }
    error!("invalid device");
    Err(())
}

/// 连接处理Handler
async fn handler(mut stream: TcpStream, hb_interval: u64, redis_cfg: RedisCfg) {
    let (mut stream_read, mut stream_write) = stream.split();
    let mut stream_reader: BufReader<ReadHalf<'_>> = BufReader::new(stream_read);
    // let mut stream_writer: BufWriter<WriteHalf<'_>> = BufWriter::new(stream_write);

    // 等待新连接40s上报sn信息，超时退出(40s来自并发测试，当瞬间发起大量连接时，从os层面无法及时将这些数据上报到应用层)
    let sn = if let Ok(Ok(v)) = timeout(Duration::from_millis(40000), handshake(&mut stream_reader)).await {
        v
    } else {
        error!("invalid sn connection");
        return;
    };

    // 创建设备
    let mut dev = device::Device::new(sn.clone());
    dev.set_heartbeat_period(Duration::from_secs(hb_interval));

    // 创建映射到redis的设备
    let mut dev2redis: map2redis::Device2redis;


    if let (Some(ip), Some(port)) = (&redis_cfg.ip, &redis_cfg.port) {
        if let Ok(v) = map2redis::Device2redis::new(dev, ip, port).await {
            dev2redis = v;
        } else {
            error!("create dev2redis instance fail");
            return;
        }
    } else {
        error!("have no redis config info");
        return;
    }

    // 激活设备，包括向redis添加设备上线信息
    println!("device: {:?}", dev2redis.dev.sn);
    if !dev2redis.activate().await {
        error!("activated device({}) fail", dev2redis.dev.sn);
        if !dev2redis.deactivate().await {
            error!("deactivated device({}) fail", dev2redis.dev.sn);
        }
        return;
    }

    // 循环处理设备消息
    loop {
        tokio::select! {
            read_up = readline(&mut stream_reader) => {
                if let Ok(v) = read_up {
                    println!("up msg: {}", v);
                }
            }
           // TODO: 这里需要优化，读redis即使没有值也是Ready状态(返回Err), 因此改async函数总是被调度
           // 所以对CPU的占用非常高?
           result = dev2redis.readline_downlink() => {
               if let Ok(msg) = result {
                   println!("down msg: {:?}", msg);
               }
           }
        }
        tokio::time::delay_for(Duration::from_millis(100)).await;
    }
}

/// 监听端口，派发连接
fn coroutines_start(ip: String, port: String, hb_interval: u64, redis_cfg: RedisCfg) -> Result<(), Box<dyn std::error::Error>> {

    // 创建调度器
    let mut rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()?;

    let addr = format!("{}:{}", ip, port);

    rt.block_on(async move {
        let mut listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                panic!("open port error: {}", e);
            }
        };

        // Create listener.
        info!("open the addr to listen {}", addr);
        let mut incoming = listener.incoming();

        // TODO 目前没有获取ip地址，将来需要绘制用户分布地图，需要ip
        // 监听端口，或许第二种loop方法更容错?
        //while let Some(stream) = incoming.next().await {
        loop {
            match incoming.next().await {
                Some(Ok(stream)) => {
                    info!("coming a connection");
                    let redis_cfg_move = redis_cfg.clone();
                    tokio::spawn(async move {
                        handler(stream, hb_interval, redis_cfg_move).await;
                    });
                }
                e => error!("{:?}", e),
            }
        }
    });

    Ok(())
}

/// 启动设备连接服务
pub fn start(perceptioncfg: PerceptCfg, rediscfg: RedisCfg) -> Result<(), ()> {
    info!("{:?}", perceptioncfg);
    println!("Device Connection Start with config {:?}", perceptioncfg);

    let ip = perceptioncfg.ip.clone().unwrap();
    let port = perceptioncfg.port.clone().unwrap();

    // 如果没有配置心跳，默认120s
    let heartbeat_interval = perceptioncfg.heartbeat_interval.unwrap_or(120);

    println!("ok: {}, {}, {:?}", ip, port, heartbeat_interval);

    match coroutines_start(ip, port, heartbeat_interval, rediscfg) {
        Ok(()) => {
            info!("start perception service success");
            Ok(())
        }
        Err(_) => {
            panic!("start perception service failed")
        }
    }
}