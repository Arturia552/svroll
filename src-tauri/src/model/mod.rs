use serde::{Deserialize, Serialize};

pub mod connect_param;
pub mod database;
pub mod db_com;

/// 后端到前端的消息实体
/// 
/// 用于从Rust后端向JavaScript前端传递各类消息和状态
#[derive(Debug, Deserialize, Serialize)]
pub struct Rs2JsEntity {
    /// 消息类型，定义消息的处理方式
    #[serde(rename = "msgType")]
    pub msg_type: Rs2JsMsgType,
    /// 消息内容
    pub msg: String,
    /// 消息时间戳
    pub time: String,
}

impl Rs2JsEntity {
    /// 创建新的消息实体
    /// 
    /// # 参数
    /// * `msg_type` - 消息类型
    /// * `msg` - 消息内容
    pub fn new(msg_type: Rs2JsMsgType, msg: String) -> Self {
        Rs2JsEntity {
            msg_type,
            msg,
            time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Rust到JavaScript的消息类型枚举
/// 
/// 定义消息在前端的展示和处理方式
#[derive(Debug, Deserialize, Serialize)]
pub enum Rs2JsMsgType {
    /// 计数器消息，用于更新UI上的计数器
    #[serde(rename = "counter")]
    Counter,
    /// 终端消息，用于在终端窗口显示
    #[serde(rename = "terminal")]
    Terminal,
    /// 错误消息，用于显示错误提示
    #[serde(rename = "error")]
    Error,
    /// 成功消息，用于显示操作成功提示
    #[serde(rename = "success")]
    Success,
    /// 连接状态消息，用于更新连接状态
    #[serde(rename = "connectState")]
    ConnectState,
}