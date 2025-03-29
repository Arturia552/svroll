use std::sync::Arc;

use dashmap::DashMap;
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

use crate::{benchmark_param::Protocol, tcp::tcp_client::TcpClient, Database, MqttClientData};

#[derive(Debug)]
pub struct AppState {
    /// MQTT客户端上下文
    mqtt_clients: DashMap<String, MqttClientData>,
    /// TCP客户端上下文
    tcp_clients: DashMap<String, (TcpClient, Option<OwnedWriteHalf>)>,
    /// 应用数据库
    database: Arc<Mutex<Database>>,
}

impl AppState {
    /// 创建新的应用状态实例
    pub fn new(database: Database) -> Self {
        Self {
            mqtt_clients: DashMap::new(),
            tcp_clients: DashMap::new(),
            database: Arc::new(Mutex::new(database)),
        }
    }

    /// 获取MQTT客户端集合引用
    pub fn mqtt_clients(&self) -> &DashMap<String, MqttClientData> {
        &self.mqtt_clients
    }

    /// 获取TCP客户端集合引用
    pub fn tcp_clients(&self) -> &DashMap<String, (TcpClient, Option<OwnedWriteHalf>)> {
        &self.tcp_clients
    }

    /// 获取数据库引用
    pub fn database(&self) -> &Arc<Mutex<Database>> {
        &self.database
    }

    /// 添加MQTT客户端
    pub fn add_mqtt_client(&self, client_id: String, client: MqttClientData) {
        self.mqtt_clients.insert(client_id, client);
    }

    /// 添加TCP客户端
    pub fn add_tcp_client(&self, client_id: String, client: (TcpClient, Option<OwnedWriteHalf>)) {
        self.tcp_clients.insert(client_id, client);
    }

    /// 获取MQTT客户端列表
    pub fn get_mqtt_client_list(&self) -> Vec<MqttClientData> {
        self.mqtt_clients
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取TCP客户端列表,只返回TcpClient
    pub fn get_tcp_client_list(&self) -> Vec<TcpClient> {
        self.tcp_clients
            .iter()
            .map(|entry| entry.value().0.clone())
            .collect()
    }

    /// 移除所有特定类型的客户端
    pub fn clear_clients(&self, client_type: Protocol) {
        match client_type {
            Protocol::Mqtt => self.mqtt_clients.clear(),
            Protocol::Tcp => self.tcp_clients.clear(),
        }
    }

    
    
}
