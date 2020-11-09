#[allow(dead_code)]
use log::{error, info, warn};
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
use crate::middleware_wrapper::json_wrapper::{
    is_heartbeat,
    MESTYPE,
    parse_msg_type,
};
use crate::perception_service::map2redis;

use super::device;

mod tcp_err {
    #[derive(Debug)]
    pub enum ServerError {
        Broken,
        INVALID_DATA,
    }
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

async fn writeline<'a>(
    stream: &'a mut BufWriter<WriteHalf<'_>>,
    line: String,
) -> Result<usize, ServerError> {
    let mut byte_line: Vec<u8> = line.trim().as_bytes().to_vec();
    if byte_line.len() == 0 {
        return Err(ServerError::INVALID_DATA);
    }
    byte_line.push('\n' as u8);
    match stream.write_all(&mut byte_line).await {
        Ok(_) => {
            stream.flush().await;
            info!("write msg -> {:?}", line);
            Ok(byte_line.len())
        }
        Err(_) => {
            info!("write broken");
            Err(ServerError::Broken)
        }
    }
}

async fn echo_pong<'a>(stream: &'a mut BufWriter<WriteHalf<'_>>) -> Result<usize, ServerError> {
    let pong_msg = "{\"type\":\"pong\"}".to_string();
    writeline(stream, pong_msg).await
}

/// 连接处理Handler
async fn handler(mut stream: TcpStream, hb_interval: u64, redis_cfg: RedisCfg) {
    let (stream_read, stream_write) = stream.split();
    let mut stream_reader: BufReader<ReadHalf<'_>> = BufReader::new(stream_read);
    let mut stream_writer: BufWriter<WriteHalf<'_>> = BufWriter::new(stream_write);

    // 等待新连接40s上报sn信息，超时退出(40s来自并发测试，当瞬间发起大量连接时，从os层面无法及时将这些数据上报到应用层)
    let sn = if let Ok(Ok(v)) = timeout(Duration::from_millis(40000), handshake(&mut stream_reader)).await {
        if let Ok(_) = echo_pong(&mut stream_writer).await {
            info!("handshake ok, from device(sn {})", v);
            v
        } else {
            error!("echo pong msg failed");
            return;
        }
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
            error!("create dev2redis instance fail, redis is running ?");
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
                if let Ok(msg) = read_up {
                    match parse_msg_type(&msg) {
                        MESTYPE::HEARTBEAT => {
                            echo_pong(&mut stream_writer).await;
                            dev2redis.dev.update_last_heartbeat_time_now();
                        }
                        MESTYPE::RAWDATA => {
                            info!("push event: {:?}", &msg);
                            dev2redis.notify_event(&msg).await;
                        }
                        MESTYPE::ACK => {
                            info!("ack type {}", &msg);
                            dev2redis.write_uplink(&msg).await;
                        }
                        MESTYPE::INVALID => {
                            println!("invalid data");
                        }
                    }

                }

            }

           result = dev2redis.readline_downlink() => {
               if let Ok(msg) = result {
                   println!("down link msg: {:?}", msg);
                   if let Ok(_) = writeline(&mut stream_writer, msg.clone()).await {
                       println!("send ok: {}", msg);
                   } else {
                       println!("send failed")
                   }
               }
           }
        }

        // 判断是否过期, 过期则设备离线
        if !dev2redis.dev.is_alive_update() {
            dev2redis.deactivate().await;
            return;
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