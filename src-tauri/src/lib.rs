pub mod benchmark_param;
pub mod context;
pub mod model;
pub mod mqtt;
pub mod tcp;
pub mod traits;

use std::sync::Arc;

pub use context::{MQTT_CLIENT_CONTEXT, TCP_CLIENT_CONTEXT};
use model::{database::Database, Rs2JsEntity};
pub use mqtt::basic::TopicWrap;
pub use mqtt::client_data::MqttClientData;
pub use mqtt::device_data::MqttSendData;
use tokio::sync::{mpsc, Mutex};

pub struct AsyncProcInputTx {
    pub inner: Mutex<mpsc::Sender<Rs2JsEntity>>,
}
