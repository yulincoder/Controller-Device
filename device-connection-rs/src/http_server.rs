use crate::connection;
use crate::device;
use actix_web::{error, get, post, web, App, Error, HttpResponse, HttpServer};
use bytes::BytesMut;
use futures::StreamExt;
use log::{info, warn};
use serde_json::json;
use std::sync::{Arc, RwLock};

#[get("/query/devices_num")]
async fn query_devices_num(
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> Result<HttpResponse, Error> {
    let resp = json!({
        "namespace": "/query/devices_num".to_string(),
        "value": format!("{}", devicepool.read().unwrap().get_devices_num()),
    });
    Ok(HttpResponse::Ok().body(resp))
}

#[get("/query/devices_alive_num")]
async fn query_devices_alive_num(
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> Result<HttpResponse, Error> {
    let resp = json!({
        "namespace": "/query/device_alive_num",
        "value": format!("{}", devicepool.read().unwrap().get_alive_devices_num()),
    });
    Ok(HttpResponse::Ok().body(resp))
}

#[get("/query/device_is_alive/{sn}")]
async fn device_is_alive(
    info: web::Path<String>,
    devicepool: web::Data<Arc<RwLock<device::DevicePool>>>,
) -> Result<HttpResponse, Error> {
    let sn = info.to_string();
    let is_alive = devicepool.write().unwrap().is_alive(&sn);

    let resp = json!({
    "namespace": "/query/device_is_alive",
    "sn": sn,
    "value": if is_alive {
            "online".to_string()
        } else {
            "offline".to_string()
        },
    });

    Ok(HttpResponse::Ok().body(resp))
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
            return Err(error::ErrorBadRequest(json!({
                "namespace": "/push/push_msg",
                "error": "overflow"
            })));
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
                return Err(error::ErrorBadRequest(json!({
                    "namespace": "/push/push_msg",
                    "error": "have no sn field"
                })));
            };
            info!("push get device({})", sn);

            let mut dsp_lock = devicepool.write().unwrap();

            if dsp_lock.is_alive(&sn) == false || dsp_lock.is_in_pool(&sn) == false {
                return Err(error::ErrorBadRequest(json!({
                    "namespace": "/push/push_msg",
                    "error": "device offline"
                })));
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
                            if let Ok(rawdata) = device.readline_timeout(5) {
                                let resp = json!({
                                    "namespace": "/push/push_msg",
                                    "value":  rawdata
                                });
                                return Ok(HttpResponse::Ok().body(resp));
                            } else {
                                return Err(error::ErrorBadRequest(json!({
                                    "namespace": "/push/push_msg",
                                    "error": "no response"
                                })));
                            }
                        }
                        Err(_) => {
                            return Err(error::ErrorBadRequest(json!({
                                "namespace": "/push/push_msg",
                                "error": "send message fail"
                            })));
                        }
                    }
                }
                _ => {
                    return Err(error::ErrorBadRequest(json!({
                          "namespace": "/push/push_msg",
                          "error": "device offline"
                    })))
                }
            }
        }
        _ => {
            return Err(error::ErrorBadRequest(json!({
                  "namespace": "/push/push_msg",
                  "error": "invalid data"
            })))
        }
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
