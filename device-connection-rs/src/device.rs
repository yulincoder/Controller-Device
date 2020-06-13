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
use crate::connection;
use crate::messagequeue::MQ;
use log::{error, info, warn};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

pub enum ReadWriteErrorCode {
    FLUSHFAIL,
    CONNECTIONBROKEN,
    TIMEOUT,
    NULLSTREAM,
    DATAINVAILD,
    NULLDATA,
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
    match writer.write_all(byte_line.as_slice()) {
        Ok(_) => {
            if flush {
                if let Err(e) = writer.flush() {
                    error!("flush error: {}", e);
                    return Err(String::from("Flush Error"));
                }
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

#[cfg(test)]
mod device_tests {
    use crate::device::Device;
    use std::sync::{Arc, RwLock};
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_device() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_device_set_get_sn() {
        let mut device: Device = Device::new("sn123".to_string());
        assert_eq!(device.get_sn(), "sn123");
        device.set_sn("sn456".to_string());
        assert_eq!(device.get_sn(), "sn456");
    }

    #[test]
    fn test_device_get_alive_time() {
        let device: Device = Device::new("sn123".to_string());
        assert_eq!(device.get_alive_time() < Duration::from_nanos(1000), true);
        sleep(Duration::from_secs(4));
        assert_eq!(device.get_alive_time() >= Duration::from_secs(4), true);
        assert_eq!(device.get_alive_time() <= Duration::from_secs(5), true);
    }

    #[test]
    fn test_is_alive() {
        let mut device: Device = Device::new("sn123".to_string());
        assert_eq!(device.is_alive(), true);
        device.deactivate();
        assert_eq!(device.is_alive(), false);
    }

    #[test]
    fn test_is_heartbeat_timeout() {
        let mut device: Device = Device::new("sn123".to_string());
        assert_eq!(device.is_heartbeat_timeout(), false);
        assert_eq!(device.is_alive(), true);
        device.set_heartbeat_period(Duration::from_secs(3));
        sleep(Duration::from_secs(4));
        assert_eq!(device.is_heartbeat_timeout(), true);
        device.update_heartbeat_timestamp_auto();
        assert_eq!(device.is_heartbeat_timeout(), false);
    }
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
            heartbeat_period: Duration::from_secs(90),
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
            heartbeat_period: Duration::from_secs(90),
        }
    }

    /// Set the period of heartbeaqt
    pub fn set_heartbeat_period(&mut self, period: Duration) {
        info!("set heartbeat to {:?}", period);
        self.heartbeat_period = period;
    }

    /// Is over the heartbeat. At time, should send the pinqqg/pong message.
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

    /// Push online to mq
    pub fn push_online_msg(&self, mq: &mut MQ) -> Result<(), ()> {
        let msg = format!("{{ \"type\": \"online\", \"sn\": \"{}\" }}", self.sn);
        match mq.push(msg.to_string().borrow()) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    /// set the status of alive
    pub fn activate(&mut self, stream: TcpStream) {
        self.is_alive = true;
        self.stream = Some(stream);
        self.last_heart_time = SystemTime::now();
    }

    // disable a device
    pub fn deactivate(&mut self) {
        warn!("deactive device({})", self.get_sn());
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
        info!("update the heartbeat timestamp {:?}", self.last_heart_time);
    }

    /// Set a Stream to device
    pub fn set_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }

    /// Response pong to device ping
    pub fn echo_pong(&mut self) -> Result<(), ()> {
        info!("ping from the device({})", self.get_sn());
        let pong_msg = "{\"type\":\"pong\"}".to_string();
        match self.writeline(pong_msg, true) {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("pong fail, deactivate the device");
                self.deactivate();
                Err(())
            }
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
                    Ok(_) => {
                        if flush {
                            if let Err(e) = writer.flush() {
                                error!("flush error with sn({}): {:?}", self.sn, e);
                                return Err(ReadWriteErrorCode::FLUSHFAIL);
                            }
                        }
                        Ok(byte_line.len())
                    }
                    _ => {
                        error!("connection broken: {}", self.sn);
                        Err(ReadWriteErrorCode::CONNECTIONBROKEN)
                    }
                }
            }
            None => {
                error!("No stream: {}", self.sn);
                Err(ReadWriteErrorCode::NULLSTREAM)
            }
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

    ///
    /// That api can only be used at Web API.
    pub fn readline_timeout(&mut self, timeout: usize) -> Result<String, ReadWriteErrorCode> {
        let mut err = ReadWriteErrorCode::NULLDATA;
        // try to read every 0.5s
        for _ in 0..(timeout * 2) {
            match self.readline() {
                Ok(msg) => match connection::parse_msg_type(&msg) {
                    connection::MESTYPE::HEARTBEAT => {
                        if let Ok(_) = self.echo_pong() {
                            info!("get the heartbeat data: {}", msg);
                            self.update_heartbeat_timestamp_auto();
                        }
                    }
                    connection::MESTYPE::RAWDATA => {
                        info!("get the rawdata: {}", msg);
                        return Ok(msg);
                    }
                    connection::MESTYPE::INVAILD => {
                        warn!("get the invalid data: {}", msg);
                    }
                },
                Err(e) => {
                    err = e;
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
        Err(err)
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
