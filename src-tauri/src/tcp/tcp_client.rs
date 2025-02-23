use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    benchmark_param::BenchmarkConfig, model::tauri_com::Task, mqtt::Client, TCP_CLIENT_CONTEXT,
};
use anyhow::{Error, Result};
use serde::Deserialize;
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedReadHalf, TcpStream},
    sync::Semaphore,
    time::Instant,
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;
use tracing::error;

use super::RequestCodec;

#[derive(Debug, Clone)]
pub struct TcpClient {
    pub send_data: Arc<Vec<u8>>,
    pub enable_register: bool,
}

impl TcpClient {
    pub fn new(send_data: Arc<Vec<u8>>, enable_register: bool) -> Self {
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

#[derive(Debug, Clone, Deserialize)]
pub struct TcpClientData {
    pub mac: String,
    pub is_connected: bool,
    pub is_register: bool,
}

impl TcpClientData {
    pub fn new(mac: String) -> Self {
        Self {
            mac,
            is_connected: false,
            is_register: false,
        }
    }

    pub fn set_mac(&mut self, mac: String) {
        self.mac = mac;
    }

    pub fn get_mac(&self) -> String {
        self.mac.clone()
    }

    pub fn set_is_connected(&mut self, is_connected: bool) {
        self.is_connected = is_connected;
    }

    pub fn get_is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn set_is_register(&mut self, is_register: bool) {
        self.is_register = is_register;
    }

    pub fn get_is_register(&self) -> bool {
        self.is_register
    }
}

impl Client<Vec<u8>, TcpClientData> for TcpClient {
    type Item = TcpClientData;

    async fn setup_clients(
        &self,
        config: &BenchmarkConfig<Vec<u8>, TcpClientData>,
    ) -> Result<Vec<TcpClientData>, Error> {
        let clients = vec![];

        let semaphore = Arc::new(Semaphore::new(config.get_max_connect_per_second()));

        for client in config.get_clients() {
            let semaphore = Arc::clone(&semaphore);
            let permit = semaphore.acquire().await.unwrap();

            let start = Instant::now();
            let conn = TcpStream::connect(config.get_broker()).await?;

            let (reader, writer) = conn.into_split();

            TCP_CLIENT_CONTEXT.insert(client.get_mac(), writer);

            tokio::spawn(async move {
                Self::process_read(reader).await;
            });

            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(1) {
                tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
            }

            drop(permit);
        }

        Ok(clients)
    }

    async fn wait_for_connections(&self, clients: &mut [Self::Item]) {
        for client in clients {
            if let Some(writer) = TCP_CLIENT_CONTEXT.get(&client.get_mac()) {
                match writer.writable().await {
                    Ok(_) => {
                        self.on_connect_success(client).await;
                    }
                    Err(e) => {
                        error!("{}", format!("连接失败: {}", e));
                    }
                }
            }
        }
    }

    async fn on_connect_success(&self, client: &mut TcpClientData) -> Result<(), Error> {
        if let Some(mut writer) = TCP_CLIENT_CONTEXT.get_mut(&client.get_mac()) {
            if self.get_enable_register() {
                // 发送注册包
                match writer.write("abc".as_bytes()).await {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                }
            }
        }
        Ok(())
    }

    async fn spawn_message(
        &self,
        clients: Vec<TcpClientData>,
        task: &Task,
        config: &BenchmarkConfig<Vec<u8>, TcpClientData>,
    ) {
        // 确定每个线程处理的客户端数量
        let startup_thread_size = clients.len() / config.thread_size
            + if clients.len() % config.thread_size != 0 {
                1
            } else {
                0
            };
        // 按线程大小将客户端分组
        let clients_group = clients.chunks(startup_thread_size);

        for group in clients_group {
            let mut groups = group.to_vec();
            let msg_value = Arc::clone(&self.send_data);
            let counter = task.counter.clone();
            let send_interval = config.send_interval;
            let enable_register = config.enable_register;

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(send_interval));
                loop {
                    interval.tick().await;
                    for client in groups.iter_mut() {
                        if let Some(mut writer) = TCP_CLIENT_CONTEXT.get_mut(&client.get_mac()) {
                            if enable_register && writer.writable().await.is_ok() {
                                let _ = writer.write(&msg_value).await;
                                counter.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                }
            });
        }
    }
}
