#![allow(dead_code)]

//use log::{error, info, warn};
use serde_json;
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum MESTYPE {
    HEARTBEAT,
    ACK,
    RAWDATA,
    // 目前只包括event消息
    INVALID,
}

/// If the msg is a heartbeat ping type, return the sn
///
/// # Examples
/// ```
/// let ok_str = r#"{"type": "ping","sn": "123"}"#;
/// assert_eq!(carte::is_heartbeat(&ok_str.to_string()), Some("123".to_string()));
/// let err_str = r#"{"type": "ping","what": "error"}"#;
/// assert_ne!(carte::is_heartbeat(&err_str.to_string()), Some("123".to_string()));
/// ```
pub fn is_heartbeat(msg: &String) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                let msg_type: Option<String> = if t.contains_key("type") {
                    match t.get("type") {
                        Some(Value::String(v)) => Some(v.to_string()),
                        _ => None,
                    }
                } else {
                    None
                };

                match msg_type {
                    Some(s) if s == "ping" => {
                        if t.contains_key("sn") {
                            match t.get("sn") {
                                Some(Value::String(v)) => Some(v.to_string()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    sn
}

#[test]
fn is_heartbeat_test() {
    let ok_str = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(is_heartbeat(&ok_str.to_string()), Some("123".to_string()));
    let err_str = r#"{"type": "ping","what": "error"}"#;
    assert_ne!(is_heartbeat(&err_str.to_string()), Some("123".to_string()));
}

pub fn parse_sn(msg: &String) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                if t.contains_key("sn") {
                    match t.get("sn") {
                        Some(Value::String(v)) => Some(v.to_string()),
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

#[test]
fn parse_sn_test() {
    let ok_ping = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(parse_sn(&ok_ping.to_string()), Some("123".to_string()));
    let ok_rawdata = r#"{"type": "rawdata","sn": "145623"}"#;
    assert_eq!(
        parse_sn(&ok_rawdata.to_string()),
        Some("145623".to_string())
    );
    let err0_sn = r#"{"type": "rawdata","snfuck": "145623"}"#;
    assert_eq!(parse_sn(&err0_sn.to_string()), None);
    let err1_sn = r#"{"what": "error"}"#;
    assert_eq!(parse_sn(&err1_sn.to_string()), None);
    let err2_sn = r#"{"what": "err"#;
    assert_eq!(parse_sn(&err2_sn.to_string()), None);
}

///
/// When the msg is a valid json and include the `type` field,
/// but not the heartbeat type, it is a raw data.
/// 只要包含type与sn，就是rawdata，acktype与heartbeat类型是rawdata的子集
pub fn is_rawdata(msg: &String) -> bool {
    if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                if t.contains_key("type") && t.contains_key("sn") {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

/// 判断是不是ack： get ack, set ack
pub fn is_ack(msg: &str) -> Option<String> {
    let sn: Option<String> = if let Ok(data) = serde_json::from_str(msg) {
        match data {
            Value::Object(t) => {
                let msg_type: Option<String> = if t.contains_key("type") {
                    match t.get("type") {
                        Some(Value::String(v)) => Some(v.to_string()),
                        _ => None,
                    }
                } else {
                    None
                };

                match msg_type {
                    Some(s) if s == "setack" || s == "getack" => {
                        if t.contains_key("sn") {
                            match t.get("sn") {
                                Some(Value::String(v)) => Some(v.to_string()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    sn
}

pub fn parse_msg_type(msg: &String) -> MESTYPE {
    if let Some(_) = is_heartbeat(msg) {
        return MESTYPE::HEARTBEAT;
    }
    if let Some(_) = is_ack(msg) {
        return MESTYPE::ACK;
    }
    if is_rawdata(msg) {
        return MESTYPE::RAWDATA;
    }
    MESTYPE::INVALID
}

#[test]
fn parse_msg_type_test() {
    let ok_ping = r#"{"type": "ping","sn": "123"}"#;
    assert_eq!(parse_msg_type(&ok_ping.to_string()), MESTYPE::HEARTBEAT);
    let ok_rawdata = r#"{"type": "rawdata","sn": "123"}"#;
    assert_eq!(parse_msg_type(&ok_rawdata.to_string()), MESTYPE::RAWDATA);
    let err1_rawdata = r#"{"what": "error"}"#;
    assert_eq!(parse_msg_type(&err1_rawdata.to_string()), MESTYPE::INVALID);
    let err2_rawdata = r#"{"what": "err"#;
    assert_eq!(parse_msg_type(&err2_rawdata.to_string()), MESTYPE::INVALID);
}
