use actix_web::{
    App,
    Error,
    error,
    get,
    HttpResponse,
    //HttpRequest,
    HttpServer,
    post,
    //Responder,
    web,
    web::BytesMut};
use futures::StreamExt;
#[allow(unused_imports)]
use log::{error, info, warn};
use serde_json::json;

use crate::common::config;
use crate::middleware::{
    json_warpper as jw,
    query_redis as qr,
};

#[get("/query/service_version")]
async fn query_service_version() -> Result<HttpResponse, Error> {
    let resp = json!({
        "namespace": "/query/http_service_version",
        "value": "v1.2.0-20201109a",
    });
    Ok(HttpResponse::Ok().body(resp))
}


#[get("/query/devices_num")]
async fn query_devices_num() -> Result<HttpResponse, Error> {
    let rv = qr::get_devices_num().await;
    let resp = if let Ok(e) = rv {
        json!({
            "namespace": "/query/devices_num".to_string(),
            "status": "200",
            "value": format!("{}", e.unwrap_or("none".to_string()))
        })
    } else {
        json!({
            "namespace": "/query/devices_num".to_string(),
            "status": "404",
            "error": "no value"
        })
    };

    Ok(HttpResponse::Ok().body(resp))
}

#[get("/query/devices_alive_num")]
async fn query_devices_alive_num() -> Result<HttpResponse, Error> {
    let rv = qr::get_alive_devices_num().await;
    let resp = if let Ok(e) = rv {
        json!({
            "namespace": "/query/device_alive_num".to_string(),
            "status": "200",
            "value": format!("{}", e.unwrap_or("none".to_string()))
        })
    } else {
        json!({
            "namespace": "/query/device_alive_num".to_string(),
            "status": "404",
            "error": "no value"
        })
    };

    Ok(HttpResponse::Ok().body(resp))
}

#[get("/query/device_is_alive/{sn}")]
async fn query_device_is_alive(
    info: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let sn = info.to_string();

    let resp = if qr::sn_is_alive(&sn).await {
        json!({
            "namespace": "/query/device_is_alive".to_string(),
            "status": "200",
            "sn": sn,
            "value": "online"
        })
    } else {
        json!({
            "namespace": "/query/device_is_alive".to_string(),
            "status": "200",
            "sn": sn,
            "value": "offline"
        })
    };

    Ok(HttpResponse::Ok().body(resp))
}

const MAX_SIZE: usize = 262_144;

#[post("/push/push_msg")]
async fn push_get(
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest(json!({
                "namespace": "/push/push_msg",
                "status": "404",
                "error": "overflow"
            })));
        }
        body.extend_from_slice(&chunk);
    }

    match String::from_utf8(body.to_vec()) {
        Ok(body_string) => {
            let sn = if let Some(sn) = jw::parse_sn(&body_string) {
                sn
            } else {
                warn!("invaild request, have no sn field");
                return Err(error::ErrorBadRequest(json!({
                    "namespace": "/push/push_msg",
                    "status": "404",
                    "error": "have no sn field"
                })));
            };
            info!("push get device({})", sn);
            if qr::sn_is_alive(&sn).await == false {
                return Err(error::ErrorBadRequest(json!({
                    "namespace": "/push/push_msg",
                    "status": "404",
                    "error": "device offline"
                })));
            }
            match qr::transparent_transmit_wit_ack(&sn, &body_string).await {
                Ok(Some(resp)) => {
                    let resp = json!({
                                    "namespace": "/push/push_msg",
                                    "status": "200",
                                    "value":  resp
                                });
                    return Ok(HttpResponse::Ok().body(resp));
                }
                Ok(None) => {
                    return Err(error::ErrorBadRequest(json!({
                                    "namespace": "/push/push_msg",
                                    "status": "404",
                                    "error": "no response"
                                })));
                }
                Err(e) => {
                    return Err(error::ErrorBadRequest(json!({
                                "namespace": "/push/push_msg",
                                "status": "408",
                                "error": format!("send message fail {}", e)
                            })));
                }
            }
        }
        _ => {
            return Err(error::ErrorBadRequest(json!({
                "namespace": "/push/push_msg",
                "error": "invalid data"
                })));
        }
    }
}

#[actix_web::main]
pub async fn launch(httpconf: config::HttpServiceConfig, _redisconf: config::RedisConfig) -> std::io::Result<()> {
    HttpServer::new(move || {
        //let s = RedisSession::new("127.0.0.1:6379", &[0; 32]);
        App::new()
            //.data(redis_conn)
            //.wrap(RedisSession::new("127.0.0.1:6379", &[0; 32]))
            .service(query_service_version)
            .service(query_devices_num)
            .service(query_devices_alive_num)
            .service(query_device_is_alive)
            .service(push_get)
    })
        .bind(format!("{}:{}", httpconf.ip.unwrap_or("0.0.0.0".to_string()),
                      httpconf.port.unwrap_or("8000".to_string())))?
        .run()
        .await
}