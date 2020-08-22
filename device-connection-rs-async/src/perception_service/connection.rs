use log::{info};

use crate::common::config::PerceptionServiceConfig as connCfg;
use crate::perception_service::device::{Device};

pub fn start(conncfg: connCfg) -> Result<(), ()> {
    info!("{:?}", conncfg);
    println!("Device Connection Start with config {:?}", conncfg);

    let mut device = Device::new("fuck".to_string());
    let (_ip, _port) = (conncfg.ip.clone().unwrap(), conncfg.port.clone().unwrap());

    info!("{:?}", device.is_alive_update());

    Ok(())
}