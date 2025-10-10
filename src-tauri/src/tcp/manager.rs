use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{Error, Result};
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedWriteHalf, TcpStream},
    sync::RwLock,
    task::JoinHandle,
    time::{sleep, Instant},
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tracing::{debug, error, info};

use crate::{
    config::BasicConfig, context::get_app_state, state::AppState, task::Task, tcp::{tcp_client::TcpSendData, RequestCodec, TcpClient}, ConnectionState
};

/// 高效的TCP客户端管理器
///
/// 通过客户端MAC地址池和轻量级句柄来管理客户端
pub struct TcpClientManager {
    /// 客户端MAC地址池，避免传递完整的客户端数据
    client_macs: Arc<Vec<String>>,
    /// 发送数据模板
    send_data: Arc<TcpSendData>,
    /// TCP连接映射表 - MAC地址到写入端的映射
    connections: Arc<RwLock<HashMap<String, OwnedWriteHalf>>>,
}

impl TcpClientManager {
    /// 创建新的TCP客户端管理器
    pub fn new(client_macs: Vec<String>, send_data: Arc<TcpSendData>) -> Self {
        Self {
            client_macs: Arc::new(client_macs),
            send_data,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_send_data(&self) -> &Arc<TcpSendData> {
        &self.send_data
    }

    /// 获取客户端MAC地址池的引用
    pub fn get_client_macs(&self) -> &Arc<Vec<String>> {
        &self.client_macs
    }

    /// 批量创建TCP客户端连接
    ///
    /// 返回成功创建的客户端MAC地址列表
    pub async fn batch_setup_clients(
        &self,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<String>, Error> {
        let mut successful_clients = Vec::new();
        let app_state = get_app_state();
        let max_conn_per_second = config.get_max_connect_per_second();
        let (tx, mut rx) = tokio::sync::mpsc::channel(config.get_clients().len());

        let mut interval =
            tokio::time::interval(Duration::from_millis(1000 / max_conn_per_second as u64));

        // 启动连接任务
        for (idx, client) in config.get_clients().iter().enumerate() {
            interval.tick().await;

            let broker = config.get_broker().to_string();
            let client_mac = client.get_mac();
            let tx = tx.clone();
            let connections = Arc::clone(&self.connections);

            tokio::spawn(async move {
                let start_time = Instant::now();
                match Self::setup_single_client(&broker, &client_mac, connections).await {
                    Ok(_) => {
                        let elapsed = start_time.elapsed();
                        if elapsed > Duration::from_secs(1) {
                            error!("TCP连接耗时过长: {:?}, 客户端: {}", elapsed, client_mac);
                        }
                        debug!("TCP客户端连接成功: {}", client_mac);
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

        // 收集连接结果
        let mut clients = config.get_clients().clone();
        while let Some((idx, success)) = rx.recv().await {
            if success {
                clients[idx].set_connection_state(ConnectionState::Connected);
                successful_clients.push(clients[idx].get_mac());
                app_state.tcp_clients().insert(
                    clients[idx].get_mac(),
                    (clients[idx].clone(), None), // writer已经移到connections中管理
                );
            } else {
                clients[idx].set_connection_state(ConnectionState::Failed);
            }
        }

        Ok(successful_clients)
    }

    /// 设置单个TCP客户端连接
    async fn setup_single_client(
        broker: &str,
        client_mac: &str,
        connections: Arc<RwLock<HashMap<String, OwnedWriteHalf>>>,
    ) -> Result<(), Error> {
        let stream = TcpStream::connect(broker).await?;
        let (reader, writer) = stream.into_split();

        // 将writer存储到连接池中
        {
            let mut conn_map = connections.write().await;
            conn_map.insert(client_mac.to_string(), writer);
        }

        // 启动读取任务
        let client_mac_clone = client_mac.to_string();
        tokio::spawn(async move {
            Self::process_read(reader, client_mac_clone).await;
        });

        Ok(())
    }

    /// 处理TCP读取数据
    async fn process_read(reader: tokio::net::tcp::OwnedReadHalf, client_mac: String) {
        let mut frame_reader = FramedRead::new(reader, RequestCodec);

        loop {
            match frame_reader.next().await {
                None => {
                    debug!("TCP客户端 {} 连接关闭", client_mac);
                    break;
                }
                Some(Err(e)) => {
                    error!("TCP客户端 {} 读取错误: {:?}", client_mac, e);
                    break;
                }
                Some(Ok(req_resp)) => {
                    debug!("收到TCP客户端 {} 的请求: {:?}", client_mac, req_resp);
                }
            }
        }

        // 清理连接
        let app_state = get_app_state();
        if let Some(mut client_data) = app_state.tcp_clients().get_mut(&client_mac) {
            client_data.0.set_connection_state(ConnectionState::Failed);
        }
    }

    /// 启动消息发送任务
    ///
    pub async fn spawn_message_tasks(
        &self,
        client_macs: Vec<String>,
        task: &Task,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> Result<Vec<JoinHandle<()>>, Error> {
        info!("开始发送TCP消息...");

        let clients_per_thread = (client_macs.len() + config.thread_size - 1) / config.thread_size;
        let mac_groups: Vec<Vec<String>> = client_macs
            .chunks(clients_per_thread)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(mac_groups.len());

        for group in mac_groups {
            let handle = self.spawn_single_message_task(group, task, config);
            handles.push(handle);
        }

        Ok(handles)
    }

    /// 启动单个消息发送任务
    fn spawn_single_message_task(
        &self,
        client_macs: Vec<String>,
        task: &Task,
        config: &BasicConfig<TcpSendData, TcpClient>,
    ) -> JoinHandle<()> {
        let send_data = Arc::clone(&self.send_data);
        let connections = Arc::clone(&self.connections);
        let counter = Arc::clone(&task.counter);
        let status = Arc::clone(&task.status);
        let send_interval = config.send_interval;

        tokio::spawn(async move {
            // 🎯 优化：在循环外部获取 app_state，避免每次发送都调用 get_app_state()
            let app_state = get_app_state();
            let mut interval = tokio::time::interval(Duration::from_secs(send_interval));

            loop {
                if !status.load(Ordering::SeqCst) {
                    info!("停止发送TCP消息");
                    break;
                }

                interval.tick().await;

                for client_mac in &client_macs {
                    if let Err(e) =
                        Self::send_single_message(client_mac, &send_data, &connections, &counter, app_state)
                            .await
                    {
                        error!("发送TCP消息失败 - 客户端MAC: {}, 错误: {:?}", client_mac, e);
                    }
                }
            }
        })
    }

    /// 发送单条TCP消息
    async fn send_single_message(
        client_mac: &str,
        send_data: &Arc<TcpSendData>,
        connections: &Arc<RwLock<HashMap<String, OwnedWriteHalf>>>,
        counter: &Arc<AtomicU32>,
        app_state: &AppState,
    ) -> Result<(), Error> {
        // 🎯 优化：使用传入的 app_state，避免重复调用 get_app_state()

        // 检查客户端状态
        let is_connected = {
            if let Some(client_data) = app_state.tcp_clients().get(client_mac) {
                client_data.0.is_connected()
            } else {
                false
            }
        };

        if !is_connected {
            return Ok(()); // 客户端未连接，跳过发送
        }

        // 获取连接写入端
        let mut conn_map = connections.write().await;
        if let Some(writer) = conn_map.get_mut(client_mac) {
            // 检查连接是否可写
            if writer.writable().await.is_ok() {
                writer.write_all(&send_data.data).await?;
                counter.fetch_add(1, Ordering::SeqCst);
            } else {
                // 连接不可写，移除连接并更新状态
                conn_map.remove(client_mac);
                if let Some(mut client_data) = app_state.tcp_clients().get_mut(client_mac) {
                    client_data.0.set_connection_state(ConnectionState::Failed);
                }
                return Err(anyhow::anyhow!("TCP连接不可写"));
            }
        } else {
            return Err(anyhow::anyhow!("未找到TCP连接"));
        }

        Ok(())
    }

    /// 等待所有客户端连接完成
    pub async fn wait_for_connections(&self, client_macs: &[String]) -> bool {
        let mut futures = Vec::with_capacity(client_macs.len());

        for client_mac in client_macs {
            let client_mac = client_mac.clone();
            futures.push(tokio::spawn(async move {
                Self::wait_single_connection(client_mac).await
            }));
        }

        let mut all_connected = true;
        for future in futures {
            match future.await {
                Ok(connected) => {
                    if !connected {
                        all_connected = false;
                    }
                }
                Err(e) => {
                    error!("等待TCP连接任务失败: {:?}", e);
                    all_connected = false;
                }
            }
        }

        all_connected
    }

    /// 等待单个客户端连接
    async fn wait_single_connection(client_mac: String) -> bool {
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100; // 10秒超时

        while attempts < MAX_ATTEMPTS {
            let app_state = get_app_state();
            if let Some(client_data) = app_state.tcp_clients().get(&client_mac) {
                if client_data.0.is_connected() {
                    return true;
                }
            }

            sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        // 连接超时，设置状态为失败
        let app_state = get_app_state();
        if let Some(mut client) = app_state.tcp_clients().get_mut(&client_mac) {
            client.0.set_connection_state(ConnectionState::Failed);
        }

        error!("TCP客户端 {} 连接超时", client_mac);
        false
    }

    /// 获取连接统计信息
    pub fn get_connection_stats(&self) -> TcpConnectionStats {
        let app_state = get_app_state();
        let mut stats = TcpConnectionStats::default();

        for client_mac in self.client_macs.iter() {
            if let Some(client_data) = app_state.tcp_clients().get(client_mac) {
                match client_data.0.get_connection_state() {
                    ConnectionState::Connected => stats.connected += 1,
                    ConnectionState::Connecting => stats.connecting += 1,
                    ConnectionState::Failed => stats.failed += 1,
                }
            }
        }

        stats.total = self.client_macs.len();
        stats
    }

    /// 关闭所有连接
    pub async fn shutdown(&self) {
        info!("关闭所有TCP连接...");

        let mut conn_map = self.connections.write().await;
        for (client_mac, mut writer) in conn_map.drain() {
            if let Err(e) = writer.shutdown().await {
                error!("关闭TCP连接失败 - 客户端MAC: {}, 错误: {:?}", client_mac, e);
            }
        }

        // 更新客户端状态
        let app_state = get_app_state();
        for client_mac in self.client_macs.iter() {
            if let Some(mut client_data) = app_state.tcp_clients().get_mut(client_mac) {
                client_data.0.set_connection_state(ConnectionState::Failed);
            }
        }

        info!("所有TCP连接已关闭");
    }
}

/// TCP连接统计信息
#[derive(Debug, Default)]
pub struct TcpConnectionStats {
    pub total: usize,
    pub connected: usize,
    pub connecting: usize,
    pub failed: usize,
}

impl TcpConnectionStats {
    /// 获取连接成功率
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.connected as f64 / self.total as f64
        }
    }

    /// 获取活跃连接数
    pub fn active_connections(&self) -> usize {
        self.connected
    }

    /// 获取失败连接数
    pub fn failed_connections(&self) -> usize {
        self.failed
    }
}

impl Clone for TcpClientManager {
    fn clone(&self) -> Self {
        Self {
            client_macs: Arc::clone(&self.client_macs),
            send_data: Arc::clone(&self.send_data),
            connections: Arc::clone(&self.connections),
        }
    }
}
