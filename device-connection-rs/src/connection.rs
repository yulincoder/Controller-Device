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

/// If the msg is a heartbeat ping type, return the sn
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

fn handle_recv_msg(device: Arc<RwLock<device::Device>>) {
    info!("ping to heartbeat");
    {
        device.write().unwrap().update_heartbeat_timestamp_auto();
    }
    {
        let mut pong_fail = false;
        if device.write().unwrap().pong_to_device() == false {
            info!("pong fail");
            pong_fail = true;
        }
        if pong_fail {
            device.write().unwrap().deactivate();
        }
    }
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
            error!("no device");
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }

    let device = Arc::new(RwLock::new(device::Device::new(pinsn.clone())));
    if let Err(_) = stream.set_nonblocking(true) {
        error!("set stream to no blocking fail");
        return;
    }
    dsp.write()
        .unwrap()
        .put_device(pinsn.clone(), device.clone());
    {
        let device_ref = dsp.write().unwrap().get_device_ref(&pinsn);
        device_ref.unwrap().write().unwrap().activate(stream);
    }
    if let Err(_) = mq.push(&pinsn) {
        error!("push a device online fail",);
        device.write().unwrap().deactivate();
        return;
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
                let msg_t: device::MESTYPE;
                {
                    msg_t = device.read().unwrap().parse_msg_type(&msg);
                }

                match msg_t {
                    device::MESTYPE::HEARTBEAT => {
                        handle_recv_msg(device.clone());
                    }
                    device::MESTYPE::RAWDATA => {
                        info!("push message to mq: {}", msg);
                        if let Err(_) = mq.push(&msg) {
                            error!("push MQ fail: {}", msg);
                        }
                    }
                    device::MESTYPE::INVAILD => {}
                    device::MESTYPE::NULL => {}
                }

                if let Err(_) = mq.push(&msg_trim) {
                    info!("fail push message to mq: {}", msg);
                }
            }
            _ => {}
        }
        {
            // Timeout
            if device.read().unwrap().is_heartbeat_timeout() {
                device.write().unwrap().deactivate();
            }
        }
        thread::sleep(Duration::from_millis(1500));
    }
}

pub fn start(devicepool: Arc<RwLock<device::DevicePool>>) {
    let mut thread_vec = vec![];
    //let mut redis_mq = Arc::new(RwLock::new(messagequeue::MQ::new("redis://127.0.0.1").unwrap()));

    info!("Start listen the port");
    let listener = if let Ok(t) = TcpListener::bind("0.0.0.0:9200") {
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
