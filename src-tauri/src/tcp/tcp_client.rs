use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    context::get_app_state, mqtt::Client, param::BasicConfig, task::Task, ConnectionState,
};
use anyhow::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedReadHalf, TcpStream},
    time::Instant,
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tracing::error;

use super::{manager::TcpClientManager, RequestCodec};

/// TCP发送数据结构
///
/// 包含要通过TCP发送的二进制数据
/// 使用Arc包装以减少克隆开销
#[derive(Debug, Clone, Deserialize)]
pub struct TcpSendData {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub data: Arc<Vec<u8>>,
}

/// 反序列化十六进制字符串为字节数组的辅助函数
///
/// # 参数
/// * `deserializer` - 反序列化器
///
/// # 返回
/// 成功返回字节数组，失败返回反序列化错误
pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = hex::decode(s)
        .map_err(|e| serde::de::Error::custom(format!("无效的十六进制字符串: {}", e)))?;
    Ok(Arc::new(bytes))
}

/// TCP客户端上下文
///
/// 管理TCP客户端配置和数据发送
#[derive(Clone)]
pub struct TcpClientContext {
    /// 客户端管理器
    manager: Arc<TcpClientManager>,
}

impl TcpClientContext {
    /// 创建新的TCP客户端上下文
    ///
    /// # 参数
    /// * `send_data` - 要发送的数据模板
    pub fn new(send_data: Arc<TcpSendData>) -> Self {
        // 预先创建空的客户端MAC列表，将在setup_clients中填充
        let manager = TcpClientManager::new(Vec::new(), send_data);
        Self {
            manager: Arc::new(manager),
        }
    }

    /// 使用客户端MAC地址列表创建上下文
    pub fn create_with_client_macs(client_macs: Vec<String>, send_data: Arc<TcpSendData>) -> Self {
        let manager = TcpClientManager::new(client_macs, send_data);
        Self {
            manager: Arc::new(manager),
        }
    }

    /// 获取发送数据的引用
    pub fn get_send_data(&self) -> &Arc<TcpSendData> {
        &self.manager.get_send_data()
    }

    /// 获取连接统计信息
    pub fn get_connection_stats(&self) -> super::manager::TcpConnectionStats {
        self.manager.get_connection_stats()
    }
}

/// TCP客户端
///
/// 表示单个TCP连接客户端
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TcpClient {
    /// 客户端MAC地址标识
    #[serde(rename = "clientId")]
    pub mac: String,
    /// 连接状态
    #[serde(default)]
    #[serde(rename = "connectionState")]
    pub connection_state: ConnectionState,
}

impl TcpClient {
    /// 设置MAC地址
    pub fn set_mac(&mut self, mac: String) {
        self.mac = mac;
    }

    /// 获取MAC地址
    pub fn get_mac(&self) -> String {
        self.mac.clone()
    }

    /// 获取连接状态
    pub fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    /// 设置连接状态
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    /// 判断是否已连接
    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }
}

/// 实现Client trait，定义TCP客户端的核心功能
impl Client<TcpSendData, TcpClient> for TcpClientContext {
    type Item = String; // 直接使用客户端MAC地址作为Item类型

    async fn setup_clients(
        &self,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<String>, Error> {
        // 使用管理器进行批量设置
        self.manager.batch_setup_clients(config).await
    }

    async fn spawn_message(
        &self,
        client_macs: Vec<String>,
        task: &Task,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Error> {
        // 使用管理器启动消息发送任务
        self.manager
            .spawn_message_tasks(client_macs, task, config)
            .await
    }

    async fn wait_for_connections(&self, client_macs: &mut [String]) -> bool {
        // 使用管理器等待连接
        self.manager.wait_for_connections(client_macs).await
    }
}
