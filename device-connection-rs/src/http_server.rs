use crate::connection;
use crate::device;
use actix_web::{error, get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use bytes::BytesMut;
use futures::StreamExt;
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

const MAX_SIZE: usize = 262_144;
#[post("/push/push_msg")]
async fn push_get(
    mut payload: web::Payload,
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    match String::from_utf8(body.to_vec()) {
        Ok(body_string) => {
            println!("data: {}", body_string);
            let sn = if let Some(sn) = connection::parse_sn(&body_string) {
                sn
            } else {
                warn!("invaild request, have no sn field");
                return Err(error::ErrorBadRequest("have no sn field"));
            };
            info!("push get device({})", sn);

            let mut dsp_lock = devicepool.write().unwrap();

            if dsp_lock.is_alive(&sn) == false || dsp_lock.is_in_pool(&sn) == false {
                return Err(error::ErrorBadRequest("device offline"));
            }

            match dsp_lock.get_device_ref(&sn) {
                Some(device_lock) => {
                    drop(dsp_lock); // unlock the devicepool lock
                    let mut device = device_lock.write().unwrap();
                    match device.writeline(body_string.clone(), true) {
                        Ok(_) => {
                            info!(
                                "send message: '{}' to device({})",
                                body_string,
                                device.get_sn()
                            );
                            if let Ok(resp) = device.readline_timeout(5) {
                                return Ok(HttpResponse::Ok().body(resp));
                            } else {
                                return Err(error::ErrorBadRequest("no response"));
                            }
                        }
                        Err(_) => {
                            return Err(error::ErrorBadRequest("send message fail"));
                        }
                    }
                }
                _ => return Err(error::ErrorBadRequest("device offline")),
            }
        }
        _ => return Err(error::ErrorBadRequest("invalid data")),
    }
}

#[actix_rt::main]
pub async fn start(devicepool: Arc<RwLock<device::DevicePool>>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(devicepool.clone())
            .service(query_devices_num)
            .service(query_devices_alive_num)
            .service(push_get)
            .service(device_is_alive)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
