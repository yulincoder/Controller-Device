#![allow(dead_code)]

use log::{error, info, warn};
use serde_json;
use serde_json::Value;

pub fn parse_sn(msg: &String) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                if t.contains_key("sn") {
                    match t.get("sn").unwrap() {
                        Value::String(v) => Some(v.to_string()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };
    sn
}