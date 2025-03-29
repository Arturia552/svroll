pub mod benchmark_param;
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
use tokio::sync::{mpsc, Mutex};
pub use model::database::Database;
pub use traits::ConnectionState;

pub struct AsyncProcInputTx {
    pub inner: Mutex<mpsc::Sender<Rs2JsEntity>>,
}
