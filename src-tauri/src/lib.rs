pub mod param;
pub mod context;
pub mod model;
pub mod mqtt;
pub mod tcp;
pub mod traits;
pub mod state;

use model::Rs2JsEntity;
pub use mqtt::basic::TopicWrap;
pub use mqtt::client_data::MqttClientData;
pub use mqtt::device_data::MqttSendData;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
pub use model::database::Database;
use tracing::info;
pub use traits::ConnectionState;

/// 异步处理输入数据的发送通道
pub struct AsyncProcInputTx {
    pub inner: Mutex<mpsc::Sender<Rs2JsEntity>>,
}

pub fn rs2js<R: tauri::Runtime>(message: Rs2JsEntity, manager: &AppHandle<R>) {
    info!(?message, "rs2js");
    let payload = serde_json::to_string(&message).unwrap();
    manager.emit("rs2js", payload).unwrap();
}