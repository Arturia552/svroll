use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Error, Result};
use rumqttc::AsyncClient;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::{param::BasicConfig, task::Task, ConnectionState, MqttSendData, TopicWrap};

use super::Client;

/// MQTT客户端句柄
///
/// 实际的客户端数据存储在全局状态中
use super::manager::{ConnectionStats, MqttClientManager};

#[derive(Clone)]
pub struct MqttClient {
    /// 客户端管理器
    manager: Arc<MqttClientManager>,
}

impl MqttClient {
    /// 创建新的MQTT客户端实例
    ///
    /// # 参数
    /// * `send_data` - 要发送的数据模板
    /// * `data_topic` - 数据发送主题配置
    pub fn new(send_data: MqttSendData, data_topic: TopicWrap) -> Self {
        let manager = MqttClientManager::new(Vec::new(), Arc::new(send_data), Arc::new(data_topic));

        MqttClient {
            manager: Arc::new(manager),
        }
    }

    pub fn get_send_data(&self) -> &Arc<MqttSendData> {
        self.manager.get_send_data()
    }

    /// 获取连接统计信息
    pub fn get_connection_stats(&self) -> ConnectionStats {
        self.manager.get_connection_stats()
    }

    /// 创建新的管理器实例
    pub fn create_with_client_ids(
        client_ids: Vec<String>,
        send_data: MqttSendData,
        data_topic: TopicWrap,
    ) -> Self {
        let manager = MqttClientManager::new(client_ids, Arc::new(send_data), Arc::new(data_topic));

        MqttClient {
            manager: Arc::new(manager),
        }
    }
}

/// 实现Client trait，定义MQTT客户端的核心功能
impl Client<MqttSendData, MqttClientData> for MqttClient {
    type Item = String; // 直接使用客户端ID作为Item类型

    async fn setup_clients(
        &self,
        config: &BasicConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<String>, Error> {
        // 使用管理器进行批量设置
        self.manager.batch_setup_clients(config).await
    }

    async fn spawn_message(
        &self,
        client_ids: Vec<Self::Item>,
        task: &Task,
        config: &BasicConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<JoinHandle<()>>, Error> {
        // 使用管理器启动消息发送任务
        self.manager
            .spawn_message_tasks(client_ids, task, config)
            .await
    }

    async fn wait_for_connections(&self, client_ids: &mut [String]) -> bool {
        // 使用管理器等待连接
        self.manager.wait_for_connections(client_ids).await
    }
}

/// MQTT客户端数据结构
///
/// 存储MQTT客户端的连接信息和状态
/// 管理单个MQTT连接实例的生命周期
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct MqttClientData {
    /// 客户端唯一标识符
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// MQTT连接用户名
    pub username: String,
    /// MQTT连接密码
    pub password: String,
    /// 标识键
    #[serde(rename = "identifyKey")]
    pub identify_key: Option<String>,
    /// 设备密钥，用于消息发布
    #[serde(skip)]
    pub device_key: String,
    /// 连接状态
    #[serde(default)]
    #[serde(rename = "connectionState")]
    pub connection_state: ConnectionState,
    /// MQTT异步客户端实例
    #[serde(skip)]
    pub client: Option<Arc<AsyncClient>>,
    /// 事件循环处理任务句柄
    #[serde(skip)]
    pub event_loop_handle: Option<Arc<RwLock<Option<JoinHandle<()>>>>>,
    /// 是否正在断开连接
    #[serde(skip)]
    pub disconnecting: Arc<AtomicBool>,
}

impl MqttClientData {
    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn get_device_key(&self) -> &str {
        &self.device_key
    }

    pub fn get_identify_key(&self) -> &Option<String> {
        &self.identify_key
    }

    pub fn set_client_id(&mut self, client_id: String) {
        self.client_id = client_id;
    }

    pub fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    pub fn set_device_key(&mut self, device_key: String) {
        self.device_key = device_key;
    }
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn set_client(&mut self, client: Option<Arc<AsyncClient>>) {
        self.client = client;
    }

    pub fn get_client(&self) -> Option<Arc<AsyncClient>> {
        self.client.clone()
    }

    /// 安全断开连接
    ///
    /// 确保只执行一次断开操作
    ///
    /// # 返回
    /// 成功断开返回Ok，失败返回错误
    pub async fn safe_disconnect(&self) -> Result<(), Error> {
        if !self.disconnecting.swap(true, Ordering::SeqCst) {
            if let Some(client) = &self.client {
                client.disconnect().await?;
            }
        }
        Ok(())
    }
}
