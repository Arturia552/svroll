use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::net::tcp::OwnedWriteHalf;

use crate::MqttClientData;

// 全局静态变量，用于存储客户端上下文
pub static MQTT_CLIENT_CONTEXT: Lazy<DashMap<String, MqttClientData>> = Lazy::new(DashMap::new);

pub static TCP_CLIENT_CONTEXT: Lazy<DashMap<String, OwnedWriteHalf>> = Lazy::new(DashMap::new);
