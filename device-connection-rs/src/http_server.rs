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

#[post("/push/get")]
async fn push_get(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
            // TODO: push the data to device!
            Ok(HttpResponse::Ok().body(body))
        }
        _ => Err(error::ErrorBadRequest("invalid data")),
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
