use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    benchmark_param::BenchmarkConfig, context::get_app_state, model::tauri_com::Task, mqtt::Client,
    ConnectionState,
};
use anyhow::{Error, Result};
use bytes::buf::Writer;
use serde::{Deserialize, Deserializer, Serialize};
use tokio::{
    io::AsyncWriteExt, net::{tcp::OwnedReadHalf, TcpStream}, sync::Semaphore, time::{sleep, Instant}
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tracing::error;

use super::RequestCodec;

#[derive(Debug, Clone, Deserialize)]
pub struct TcpSendData {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub data: Vec<u8>,
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = hex::decode(s)
        .map_err(|e| serde::de::Error::custom(format!("无效的十六进制字符串: {}", e)))?;
    Ok(bytes)
}

#[derive(Debug, Clone)]
pub struct TcpClientContext {
    pub send_data: Arc<TcpSendData>,
    pub enable_register: bool,
}

impl TcpClientContext {
    pub fn new(send_data: Arc<TcpSendData>, enable_register: bool) -> Self {
        Self {
            send_data,
            enable_register,
        }
    }

    pub fn get_enable_register(&self) -> bool {
        self.enable_register
    }

    pub fn set_enable_register(&mut self, enable_register: bool) {
        self.enable_register = enable_register
    }

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TcpClient {
    #[serde(rename = "clientId")]
    pub mac: String,
    #[serde(default)]
    #[serde(rename = "connectionState")]
    pub connection_state: ConnectionState,
    #[serde(skip)]
    pub is_register: bool,
}

impl TcpClient {
    pub fn set_mac(&mut self, mac: String) {
        self.mac = mac;
    }

    pub fn get_mac(&self) -> String {
        self.mac.clone()
    }

    // 添加新方法获取连接状态
    pub fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    // 添加新方法设置连接状态
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    // 添加辅助方法判断是否已连接
    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    pub fn set_is_register(&mut self, is_register: bool) {
        self.is_register = is_register;
    }

    pub fn get_is_register(&self) -> bool {
        self.is_register
    }
}

impl Client<TcpSendData, TcpClient> for TcpClientContext {
    type Item = TcpClient;

    async fn setup_clients(
        &self,
        config: &BenchmarkConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<TcpClient>, Error> {
        let mut clients = config.get_clients().clone();
        let app_state = get_app_state();
        let max_conn_per_second = config.get_max_connect_per_second();
        let (tx, mut rx) = tokio::sync::mpsc::channel(clients.len());

        // 使用令牌桶算法控制连接速率
        let mut interval =
            tokio::time::interval(Duration::from_millis(1000 / max_conn_per_second as u64));

        for (idx, client) in clients.iter().enumerate() {
            interval.tick().await; // 等待下一个令牌

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

                        // 记录连接耗时
                        let elapsed = start_time.elapsed();
                        if elapsed > Duration::from_secs(1) {
                            error!("TCP连接耗时过长: {:?}, 客户端: {}", elapsed, client_mac);
                        }

                        // 发送连接成功信号
                        let _ = tx.send((idx, true)).await;
                    }
                    Err(e) => {
                        error!("TCP连接失败: {}, 客户端: {}", e, client_mac);
                        // 发送连接失败信号
                        let _ = tx.send((idx, false)).await;
                    }
                }
            });
        }

        drop(tx);

        // 处理连接结果
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
                    // 发送注册包
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
        config: &BenchmarkConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Error> {
        let app_state = get_app_state();
        // 确定每个线程处理的客户端数量
        let startup_thread_size = clients.len() / config.thread_size
            + if clients.len() % config.thread_size != 0 {
                1
            } else {
                0
            };
        // 按线程大小将客户端分组
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
