//!# Device and Device Pool, in which read/write opreation between threads safety
//!### Example
//! ```
//!     let devicepool = Arc::new(RwLock::new(device::DevicePool::new()));
//!     let mut thread_vec = vec![];
//!     for sn_idx in 0..3000 {
//!         let devicepool_clone = devicepool.clone();
//!         let t = thread::spawn(move || {
//!             let sn = format!("sn/{}", sn_idx);
//!             let arc_d = Arc::new(RwLock::new(device::Device::new(sn.clone())));
//!
//!             {
//!                 devicepool_clone
//!                     .write()
//!                     .unwrap()
//!                    .put_device(sn.clone(), arc_d.clone());
//!                 let dsp = devicepool_clone.read().unwrap();
//!                 let dn = dsp.get_devices_num();
//!                 info!("thread: {} dn = {}", sn_idx, dn);
//!             }
//!             {
//!                 let mut dsp = devicepool_clone.write().unwrap();
//!                 let advice_lock = dsp.get_device_ref(&sn).unwrap();
//!                 let adevice = advice_lock.read().unwrap();
//!                 info!("sn: {}, is alive, {}", (*adevice).is_alive(), (*adevice).get_sn());
//!             }
//!         });
//!
//!         thread_vec.push(t);
//!     }
//!
//!     for t in thread_vec {
//!         t.join().unwrap();
//!     }
//!     info!("pool len: {}", devicepool.read().unwrap().get_devices_num());
//!     info!(
//!         "alive pool len: {}",
//!         devicepool.read().unwrap().get_alive_devices_num()
//!     );
//! ```
#![allow(dead_code)]
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

pub enum ReadWriteErrorCode {
    FLUSHFAIL,
    CONNECTIONBROKEN,
    TIMEOUT,
    NULLSTREAM,
    DATAINVAILD,
    NULLDATA,
}

pub enum MESTYPE {
    NULL,
    HEARTBEAT,
    RAWDATA,
    INVAILD,
}

pub fn get_sn_from_string(info: String) -> String {
    String::from(info)
}

/// ## Read line from a TCPStream
/// Return the line or a Err
pub fn read_line(stream: &TcpStream) -> Result<String, String> {
    let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
    let mut recv_str = String::new();
    match reader.read_line(&mut recv_str) {
        Ok(v) => {
            if v > 0 {
                Ok(recv_str)
            } else {
                Err(String::from("NULL"))
            }
        }
        _ => Err(String::from("Read Fail")),
    }
}

/// ## Write a string line to TcpStream
/// If the the end of param String is not `\n`, will pad it to the end.
///
/// It cannot raise the error resulted in Socket peer broken.
pub fn write_line(stream: &TcpStream, line: String, flush: bool) -> Result<usize, String> {
    let mut writer: BufWriter<&TcpStream> = BufWriter::new(&stream);
    let mut byte_line: Vec<u8> = line.as_bytes().to_vec();
    if byte_line[byte_line.len() - 1] != 0xa {
        byte_line.push(0xa);
    }
    error!("卡死3？");
    match writer.write_all(byte_line.as_slice()) {
        Ok(_) => {
            error!("卡死4？");
            if flush {
                error!("卡死5？");
                if let Err(e) = writer.flush() {
                    error!("卡死6？");
                    error!("flush error: {}", e);
                    return Err(String::from("Flush Error"));
                }
                error!("卡死6？");
            }
            Ok(byte_line.len())
        }
        _ => Err(String::from("Error")),
    }
}

/// ## Device bind to a unique sn(Serial Number)
/// ```
/// let sn = String::from("12/132435");
/// let d = device::Device::new(sn.clone());
/// ```
pub struct Device {
    /// Unique serial number
    pub sn: String,

    /// A Tcp stream will be keep as a long connection.
    pub stream: Option<TcpStream>,

    /// the time of latest connection coming.
    born_time: SystemTime,

    /// Whether keep online.
    is_alive: bool,

    /// The time of last heartbeat to device.
    last_heart_time: SystemTime,

    /// The heartbeat period.
    heartbeat_period: Duration,
}

impl Device {
    /// Create a Device with it's sn
    pub fn new(sn: String) -> Device {
        Device {
            sn: sn,
            stream: None,
            born_time: SystemTime::now(),
            is_alive: true,
            last_heart_time: SystemTime::now(),
            heartbeat_period: Duration::from_secs(30),
        }
    }

    /// Create a Device with it's sn and born time
    pub fn new_with_sn_borntime(sn: String, born_time: SystemTime) -> Device {
        Device {
            sn: sn,
            stream: None,
            born_time: born_time,
            is_alive: true,
            last_heart_time: born_time.clone(),
            heartbeat_period: Duration::from_secs(30),
        }
    }

    /// Set the period of heartbeaqt
    pub fn set_heartbeat_period(&mut self, period: Duration) {
        info!("set heartbeat to {:?}", period);
        self.heartbeat_period = period;
    }

    /// Is over the heartbeat. At time, should send the ping/pong message.
    pub fn is_heartbeat_timeout(&self) -> bool {
        if let Ok(elapsed) = self.last_heart_time.elapsed() {
            elapsed > self.heartbeat_period
        } else {
            info!(
                "The last heartbeat time is invaild: {:?}",
                self.last_heart_time
            );
            panic!("The last heartbeat time is invaild")
        }
    }

    /// Query the device whether alive.
    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    /// set the status of alive
    pub fn activate(&mut self, stream: TcpStream) {
        self.is_alive = true;
        self.stream = Some(stream);
        self.last_heart_time = SystemTime::now();
    }

    // disable a device
    pub fn deactivate(&mut self) {
        self.is_alive = false;
        self.stream = None;
    }

    /// Get the sn
    pub fn get_sn(&self) -> &str {
        &self.sn
    }

    /// Set a sn
    pub fn set_sn(&mut self, sn: String) {
        self.sn = sn;
    }

    /// Get the time of born of the device
    pub fn get_born_time(&self) -> SystemTime {
        self.born_time
    }

    /// Set the time of born of the device
    pub fn set_born_time(&mut self, born_time: SystemTime) {
        self.born_time = born_time;
    }

    /// get the alive time
    pub fn get_alive_time(&self) -> Duration {
        self.born_time.elapsed().unwrap()
    }

    /// Update the last heartbeat time
    pub fn update_heartbeat_timestamp(&mut self, time: SystemTime) {
        self.last_heart_time = time;
    }

    ///  Update the last heartbeat time with auto padding
    pub fn update_heartbeat_timestamp_auto(&mut self) {
        self.update_heartbeat_timestamp(SystemTime::now());
    }

    /// Set a Stream to device
    pub fn set_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }

    /// TODO: do it.
    pub fn is_heartbeat_msg(&self, msg: &String) -> bool {
        true
    }

    /// TODO: do it.
    pub fn parse_msg_type(&self, msg: &String) -> MESTYPE {
        if self.is_heartbeat_msg(msg) {
            MESTYPE::HEARTBEAT
        } else {
            MESTYPE::RAWDATA
        }
    }

    /// Response pong to device ping
    pub fn pong_to_device(&mut self) -> bool {
        let pong_msg = "{\"pong\":\"pong\"}".to_string();
        match self.writeline(pong_msg, true) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// ## Write a string line to TcpStream
    /// If the the end of param String is not `\n`, will pad it to the end.
    ///
    /// It cannot raise the error resulted in Socket peer broken.
    pub fn writeline(&mut self, msgline: String, flush: bool) -> Result<usize, ReadWriteErrorCode> {
        let mut byte_line: Vec<u8> = msgline.as_bytes().to_vec();

        // Padding the `\n` to the end of line
        if byte_line[byte_line.len() - 1] != 0xa {
            byte_line.push(0xa);
        }

        match &self.stream {
            Some(stream) => {
                let mut writer: BufWriter<&TcpStream> = BufWriter::new(&stream);
                match writer.write_all(byte_line.as_slice()) {
                    Ok(o) => {
                        info!("writeline: {:?}", o);
                        if flush {
                            if let Err(e) = writer.flush() {
                                error!("flush error with sn({}): {:?}", self.sn, e);
                                return Err(ReadWriteErrorCode::FLUSHFAIL);
                            }
                        }
                        Ok(byte_line.len())
                    }
                    _ => Err(ReadWriteErrorCode::CONNECTIONBROKEN),
                }
            }
            None => Err(ReadWriteErrorCode::NULLSTREAM),
        }
    }

    /// ## Read line from a TCPStream
    /// Return the line or a Err
    pub fn readline(&mut self) -> Result<String, ReadWriteErrorCode> {
        match &self.stream {
            Some(stream) => {
                let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
                let mut recv_str = String::new();
                match reader.read_line(&mut recv_str) {
                    Ok(v) => {
                        if v > 0 {
                            Ok(recv_str)
                        } else {
                            // warn!("no data can be read with sn({})", self.sn);
                            Err(ReadWriteErrorCode::NULLDATA)
                        }
                    }
                    _ => {
                        // warn!("stream may be close by peer device with sn({})", self.sn);
                        Err(ReadWriteErrorCode::CONNECTIONBROKEN)
                    }
                }
            }
            None => Err(ReadWriteErrorCode::NULLSTREAM),
        }
    }
}

/// # DevicePool to maintain those TCP Connection between Server and devices
/// ```
/// let mut dsp = device::DevicePool::new();
/// let sn = String::from("12/132436");
/// dsp.put_device(sn.clone(), device::Device::new(sn.clone()));
/// ```
pub struct DevicePool {
    devices: HashMap<String, Arc<RwLock<Device>>>,
}

impl DevicePool {
    /// Create a DevicePool instance.
    pub fn new() -> DevicePool {
        DevicePool {
            devices: HashMap::new(),
        }
    }

    /// Get the number of total device in the DevicePool
    pub fn get_devices_num(&self) -> usize {
        self.devices.len().into()
    }

    /// Get the number of alive device in the DevicePool
    pub fn get_alive_devices_num(&self) -> usize {
        let mut alive_cnt: usize = 0;
        for (_k, v) in self.devices.iter() {
            alive_cnt = if v.read().unwrap().is_alive {
                alive_cnt + 1
            } else {
                alive_cnt
            }
        }
        alive_cnt
    }

    /// Query a device whether alive by sn
    pub fn is_alive(&mut self, sn: &String) -> bool {
        match self.get_device_ref(sn) {
            Some(d) => d.read().unwrap().is_alive(),
            _ => false,
        }
    }

    /// Query a device in the pool
    pub fn is_in_pool(&self, sn: &String) -> bool {
        self.devices.contains_key(sn)
    }

    /// Get the reference of device specified by sn
    pub fn get_device_ref(&mut self, sn: &String) -> Option<Arc<RwLock<Device>>> {
        if self.get_devices_num() == 0 {
            None
        } else {
            if let Some(d) = self.devices.get_mut(sn) {
                Some(d.deref().clone())
            } else {
                None
            }
        }
    }

    /// Put a device with sn to the DevicePool
    pub fn put_device(&mut self, sn: String, device: Arc<RwLock<Device>>) {
        self.devices.insert(sn, device);
    }
}
