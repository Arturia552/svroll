pub mod config;
pub mod context;
pub mod model;
pub mod mqtt;
pub mod param;
pub mod state;
pub mod task;
pub mod tcp;
pub mod traits;
pub mod utils;

pub use model::database::Database;
use model::Rs2JsEntity;
pub use model::Rs2JsMsgType;
pub use mqtt::basic::TopicWrap;
pub use mqtt::client_data::MqttClientData;
pub use mqtt::device_data::MqttSendData;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tracing::info;
pub use traits::ConnectionState;

/// 异步处理输入数据的发送通道
pub struct AsyncProcInputTx {
    pub inner: Mutex<mpsc::Sender<Rs2JsEntity>>,
}

/// 向前端发送消息
///
/// 将消息序列化并发送到前端
pub fn rs2js<R: tauri::Runtime>(message: Rs2JsEntity, manager: &AppHandle<R>) {
    info!(?message, "rs2js");
    let payload = serde_json::to_string(&message).unwrap();
    manager.emit("rs2js", payload).unwrap();
}
