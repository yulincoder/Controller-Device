use crate::device;
use actix_web::{get, web, App, HttpServer, Responder};
use log::{error, info, warn};
use std::sync::{Arc, RwLock};

#[get("/query/devices_num")]
async fn query_devices_num(
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> impl Responder {
    format!("device: {}", devicepool.read().unwrap().get_devices_num())
}

#[get("/query/devices_alive_num")]
async fn query_devices_alive_num(
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> impl Responder {
    format!(
        "device: {}",
        devicepool.read().unwrap().get_alive_devices_num()
    )
}

#[get("/query/device_is_alive/{sn}")]
async fn device_is_alive(
    info: web::Path<String>,
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> impl Responder {
    let sn = info;
    let is_alive = devicepool.write().unwrap().is_alive(&sn);
    let resp = format!("sn {}, is alive {}", sn, is_alive);
    info!("{}", resp);
    resp
}

#[actix_rt::main]
pub async fn start(devicepool: Arc<RwLock<device::DevicePool>>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(devicepool.clone())
            .service(query_devices_num)
            .service(query_devices_alive_num)
            .service(device_is_alive)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
