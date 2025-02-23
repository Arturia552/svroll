use serde::{Deserialize, Serialize};

pub mod connect_param;
pub mod tauri_com;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rs2JsEntity {
    #[serde(rename = "msgType")]
    pub msg_type: Rs2JsMsgType,
    pub msg: String,
    pub time: String,
}

impl Rs2JsEntity {
    pub fn new(msg_type: Rs2JsMsgType, msg: String) -> Self {
        Rs2JsEntity {
            msg_type,
            msg,
            time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Rs2JsMsgType {
    #[serde(rename = "counter")]
    Counter,
    #[serde(rename = "terminal")]
    Terminal,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "success")]
    Success,
}