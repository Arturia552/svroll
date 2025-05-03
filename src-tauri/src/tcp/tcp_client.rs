use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    context::get_app_state, mqtt::Client, param::BasicConfig, task::Task, ConnectionState
};
use anyhow::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::{
    io::AsyncWriteExt, net::{tcp::OwnedReadHalf, TcpStream}, time::Instant
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tracing::error;

use super::RequestCodec;

/// TCP发送数据结构
/// 
/// 包含要通过TCP发送的二进制数据
#[derive(Debug, Clone, Deserialize)]
pub struct TcpSendData {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub data: Vec<u8>,
}

/// 反序列化十六进制字符串为字节数组的辅助函数
/// 
/// # 参数
/// * `deserializer` - 反序列化器
/// 
/// # 返回
/// 成功返回字节数组，失败返回反序列化错误
pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = hex::decode(s)
        .map_err(|e| serde::de::Error::custom(format!("无效的十六进制字符串: {}", e)))?;
    Ok(bytes)
}

/// TCP客户端上下文
/// 
/// 管理TCP客户端配置和数据发送
#[derive(Debug, Clone)]
pub struct TcpClientContext {
    /// 要发送的数据
    pub send_data: Arc<TcpSendData>,
    /// 是否启用注册流程
    pub enable_register: bool,
}

impl TcpClientContext {
    /// 创建新的TCP客户端上下文
    /// 
    /// # 参数
    /// * `send_data` - 要发送的数据模板
    /// * `enable_register` - 是否启用注册流程
    pub fn new(send_data: Arc<TcpSendData>, enable_register: bool) -> Self {
        Self {
            send_data,
            enable_register,
        }
    }

    /// 获取是否启用注册流程
    pub fn get_enable_register(&self) -> bool {
        self.enable_register
    }

    /// 设置是否启用注册流程
    pub fn set_enable_register(&mut self, enable_register: bool) {
        self.enable_register = enable_register
    }

    /// 处理读取的数据
    /// 
    /// # 参数
    /// * `reader` - TCP流的读取端
    async fn process_read(reader: OwnedReadHalf) {
        let mut frame_reader = FramedRead::new(reader, RequestCodec);
        loop {
            match frame_reader.next().await {
                None => {
                    break;
                }
                Some(Err(_e)) => {
                    break;
                }
                Some(Ok(req_resp)) => {
                    println!("Received request: {:?}", req_resp);
                }
            }
        }
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
    /// 是否已完成注册
    #[serde(skip)]
    pub is_register: bool,
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

    /// 设置是否已注册
    pub fn set_is_register(&mut self, is_register: bool) {
        self.is_register = is_register;
    }

    /// 获取是否已注册状态
    pub fn get_is_register(&self) -> bool {
        self.is_register
    }
}

/// 实现Client trait，定义TCP客户端的核心功能
impl Client<TcpSendData, TcpClient> for TcpClientContext {
    type Item = TcpClient;

    async fn setup_clients(
        &self,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<TcpClient>, Error> {
        let mut clients = config.get_clients().clone();
        let app_state = get_app_state();
        let max_conn_per_second = config.get_max_connect_per_second();
        let (tx, mut rx) = tokio::sync::mpsc::channel(clients.len());

        let mut interval =
            tokio::time::interval(Duration::from_millis(1000 / max_conn_per_second as u64));

        for (idx, client) in clients.iter().enumerate() {
            interval.tick().await;

            let broker = config.broker.clone();
            let client_mac = client.get_mac();
            let tx = tx.clone();

            let start_time = Instant::now();

            tokio::spawn(async move {
                match TcpStream::connect(broker).await {
                    Ok(conn) => {
                        let (reader, writer) = conn.into_split();
                        let tcp_client = TcpClient {
                            mac: client_mac.clone(),
                            connection_state: ConnectionState::Connected,
                            is_register: false,
                        };
                        app_state
                            .tcp_clients()
                            .insert(client_mac.clone(), (tcp_client, Some(writer)));
                        tokio::spawn(async move {
                            Self::process_read(reader).await;
                        });

                        let elapsed = start_time.elapsed();
                        if elapsed > Duration::from_secs(1) {
                            error!("TCP连接耗时过长: {:?}, 客户端: {}", elapsed, client_mac);
                        }

                        let _ = tx.send((idx, true)).await;
                    }
                    Err(e) => {
                        error!("TCP连接失败: {}, 客户端: {}", e, client_mac);
                        let _ = tx.send((idx, false)).await;
                    }
                }
            });
        }

        drop(tx);

        while let Some((idx, success)) = rx.recv().await {
            if success {
                clients[idx].set_connection_state(ConnectionState::Connected);
            } else {
                clients[idx].set_connection_state(ConnectionState::Failed);
            }
        }

        Ok(clients)
    }

    async fn wait_for_connections(&self, clients: &mut [Self::Item]) {
        for client in clients {
            let _ = self.on_connect_success(client).await;
        }
    }

    async fn on_connect_success(&self, client: &mut TcpClient) -> Result<(), Error> {
        let app_state = get_app_state();
        if let Some(mut client_ref) = app_state.tcp_clients().get_mut(&client.get_mac()) {
            if let Some(writer) = client_ref.1.as_mut() {
                if self.get_enable_register() {
                    match writer.write("abc".as_bytes()).await {
                        Ok(_) => todo!(),
                        Err(_) => todo!(),
                    }
                }
            }
        }
        Ok(())
    }

    async fn spawn_message(
        &self,
        clients: Vec<TcpClient>,
        task: &Task,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Error> {
        let app_state = get_app_state();
        let startup_thread_size = clients.len() / config.thread_size
            + if clients.len() % config.thread_size != 0 {
                1
            } else {
                0
            };
        let clients_group = clients.chunks(startup_thread_size);
        let mut hanldes = vec![];

        for group in clients_group {
            let mut groups = group.to_vec();
            let msg_value = Arc::clone(&self.send_data);
            let counter = task.counter.clone();
            let send_interval = config.send_interval;

            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(send_interval));
                loop {
                    interval.tick().await;
                    for client in groups.iter_mut() {
                        if let Some(mut client_ref) =
                            app_state.tcp_clients().get_mut(&client.get_mac())
                        {
                            if let Some(writer) = client_ref.1.as_mut() {
                                if writer.writable().await.is_ok() {
                                    let _ = writer.write_all(&msg_value.data).await;
                                    counter.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                }
            });
            hanldes.push(handle);
        }
        Ok(hanldes)
    }
}
