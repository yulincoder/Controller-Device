use log::info;
use std::sync::{Arc, RwLock};
use std::thread;

mod connection;
mod device;
mod http_server;
mod loge_cfg;
mod messagequeue;

fn init() -> Result<(), String> {
    loge_cfg::log_init(Some("info"))?;
    Ok(())
}

fn main() {
    init().unwrap();

    let devicepool = Arc::new(RwLock::new(device::DevicePool::new()));

    let devicepool_clone_1 = devicepool.clone();
    let connection_server_thread = thread::spawn(move || {
        connection::start(devicepool_clone_1);
    });

    let devicepool_clone_2 = devicepool.clone();
    let web_server_thread = thread::spawn(move || match http_server::start(devicepool_clone_2) {
        Err(_) => panic!("Http start fail"),
        _ => {}
    });

    if connection_server_thread.join().is_ok() {
        info!("Connection Service Launch OK");
    }
    if web_server_thread.join().is_ok() {
        info!("Http service launch OK");
    }
}
