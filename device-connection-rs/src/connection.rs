#![allow(dead_code)]
use log::{error, info, warn};
use serde_json;
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::device;
use crate::messagequeue;

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

pub fn parse_sn(msg: &String) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
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
    } else {
        None
    };
    sn
}

#[test]
fn parse_sn_test() {
    let ok_ping = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(parse_sn(&ok_ping.to_string()), Some("123".to_string()));
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
        return MESTYPE::HEARTBEAT;
    }
    if is_rawdata(msg) {
        return MESTYPE::RAWDATA;
    }
    MESTYPE::INVAILD
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

fn handle_client(stream: TcpStream, dsp: Arc<RwLock<device::DevicePool>>) {
    let mut mq = if let Ok(mq) = messagequeue::MQ::new("redis://127.0.0.1") {
        mq
    } else {
        panic!("MQ Create Failed")
    };
    let mut pinsn: String = "null".to_string();

    for poll in 0..4 {
        if let Ok(msg) = device::read_line(&stream) {
            if let Some(sn) = is_heartbeat(&msg) {
                info!("sn {}", sn);
                pinsn = sn.clone().trim().to_string();
                break;
            }
        } else if poll == 3 {
            error!("invalid device");
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }

    let device = Arc::new(RwLock::new(device::Device::new(pinsn.clone())));
    if let Err(_) = stream.set_nonblocking(true) {
        error!("set stream to no blocking fail");
        return;
    }
    {
        dsp.write()
            .unwrap()
            .put_device(pinsn.clone(), device.clone());
    }
    {
        let device_ref = dsp.write().unwrap().get_device_ref(&pinsn);
        device_ref.unwrap().write().unwrap().activate(stream);
    }

    {
        let mut device_unlock = device.write().unwrap();
        if let Err(_) = device_unlock.echo_pong() {
            error!("echo pong fail");
            device_unlock.deactivate();
            return;
        }
        if let Err(_) = device_unlock.push_online_msg(&mut mq) {
            error!("push a device online fail",);
            device_unlock.deactivate();
            return;
        }
    }

    info!("new deivce sn: {}", pinsn);

    loop {
        // Can send message to device through MQ
        if let Ok(msg) = mq.pop_sn(&pinsn) {
            if let Err(_) = device.write().unwrap().writeline(msg.clone(), true) {
                error!("send success error");
            } else {
                info!("send a msg: {}", msg)
            }
        }

        // Receive a message from device then push to mq
        let msg_result = device.write().unwrap().readline();
        match msg_result {
            Ok(msg) => {
                info!(
                    "recv data form device({}) : {}, is alive: {}",
                    pinsn,
                    msg,
                    device.write().unwrap().is_alive()
                );
                let msg_trim = msg.trim().to_string();
                // Heartbeat message, update the ping timestamp.
                let msg_t = parse_msg_type(&msg_trim);
                match msg_t {
                    MESTYPE::HEARTBEAT => {
                        let mut device_lock = device.write().unwrap();
                        if let Err(_) = device_lock.echo_pong() {
                            warn!("echo pong the device fail");
                        } else {
                            device_lock.update_heartbeat_timestamp_auto();
                        }
                    }
                    MESTYPE::RAWDATA => {
                        info!("push message to mq: {}", msg);
                        device.write().unwrap().update_heartbeat_timestamp_auto();
                        if let Err(_) = mq.push(&msg_trim) {
                            error!("push MQ fail: {}", msg);
                        }
                    }
                    MESTYPE::INVAILD => {
                        warn!(
                            "invalid message from device({})",
                            device.read().unwrap().get_sn()
                        );
                    }
                }
            }
            _ => {}
        }
        {
            // Timeout
            if device.read().unwrap().is_alive() {
                if device.read().unwrap().is_heartbeat_timeout() {
                    warn!("device({}) offline", device.read().unwrap().get_sn());
                    device.write().unwrap().deactivate();
                    return;
                }
            } else {
                warn!(
                    "device({}) already offline",
                    device.read().unwrap().get_sn()
                );
                device.write().unwrap().deactivate();
                return;
            }
        }
        thread::sleep(Duration::from_millis(1500));
    }
}

pub fn start(devicepool: Arc<RwLock<device::DevicePool>>) {
    let mut thread_vec = vec![];
    //let mut redis_mq = Arc::new(RwLock::new(messagequeue::MQ::new("redis://127.0.0.1").unwrap()));

    info!("Start listen the port");
    let listener = if let Ok(t) = TcpListener::bind("0.0.0.0:8900") {
        t
    } else {
        error!("Open port failed");
        panic!("Open port failed")
    };

    // listener.set_nonblocking(true).expect("Cannot set non-blocking");
    listener.take_error().expect("Error occur");

    info!("{:?}", listener);

    for stream in listener.incoming() {
        match stream {
            // Spawn a thread to handle the connection
            Ok(s) => {
                let devicepool_clone = devicepool.clone();

                let t = thread::spawn(move || {
                    handle_client(s, devicepool_clone);
                });

                thread_vec.push(t);
            }
            Err(ref e) => {
                error!("Unknow connection was failed: {}", e);
            }
        }
    }

    for t in thread_vec {
        error!("Never reach to here");
        t.join().unwrap();
    }
}
