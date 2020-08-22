use log::{
    info
};
#[allow(unused_imports)]
use tokio::{
    net::TcpListener,
    net::TcpStream,
    stream::StreamExt,
    time};

use std::time::{
    SystemTime,
    Duration,
    SystemTimeError
};

pub struct Device {
    pub sn: String,
    pub stream: Option<TcpStream>,
    born_time: SystemTime,
    alive: bool,
    last_heartbeat_time: SystemTime,
    heartbeat_period: Duration,
}


#[allow(dead_code)]
impl Device {
    pub fn new(sn: String) -> Device {
        Device {
            sn,
            stream: None,
            born_time: SystemTime::now(),
            alive: true,
            last_heartbeat_time: SystemTime::now(),
            heartbeat_period: Duration::from_secs(120)}
    }

    pub fn set_heartbeat_period(&mut self, period: Duration) {
        info!("set device({}) heartbeat period to {:?}", self.sn, period);
        self.heartbeat_period = period;
    }

    pub fn update_last_heartbeat_time(&mut self, time: SystemTime) {
        self.last_heartbeat_time = time;
    }

    pub fn update_last_heartbeat_time_now(&mut self) {
        self.update_last_heartbeat_time(SystemTime::now());
    }

    // the up-to-date at calling, then update the device state.
    // So, You might get a different value for each call
    pub fn is_alive_update(&mut self) -> bool {
        self.alive = if let Ok(elapsed) = self.last_heartbeat_time.elapsed() {
            elapsed < self.heartbeat_period
        } else {
            self.alive
        };
        self.alive
    }

    pub fn is_alive_const(&self) -> bool {
        self.alive
    }

    pub fn online_time(&self) -> Result<Duration, SystemTimeError> {
        self.born_time.elapsed()
    }
}


#[cfg(test)]
mod device_test {
    use crate::perception_service::device::Device;
    use std::thread;
    use tokio::time::Duration;

    #[test]
    fn test_is_alive_update() {
        let mut device =  Device::new("D81234545".to_string());
        device.set_heartbeat_period(Duration::from_secs(3));
        thread::sleep(Duration::from_secs(2));
        assert_eq!(true, device.is_alive_update());
        thread::sleep(Duration::from_secs(2));
        assert_eq!(false, device.is_alive_update());
    }

    #[test]
    fn test_online_time() {
        let device =  Device::new("D81234545".to_string());
        thread::sleep(Duration::from_secs(2));
        assert_eq!(true,
                   device.online_time().unwrap() < Duration::from_millis(2050)
                   && device.online_time().unwrap() > Duration::from_millis(1950));
    }
}