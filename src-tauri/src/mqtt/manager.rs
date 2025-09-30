use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{Error, Result};
use tokio::{
    sync::{RwLock, Semaphore},
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, error, info};

use crate::{
    config::BasicConfig,
    context::get_app_state,
    mqtt::{client_data::MqttClientData, device_data::process_fields},
    state::AppState,
    task::Task,
    MqttSendData, TopicWrap,
};

/// 高效的MQTT客户端管理器
///
pub struct MqttClientManager {
    /// 客户端ID池，避免传递完整的客户端数据
    client_ids: Arc<Vec<String>>,
    /// 发送数据模板
    send_data: Arc<MqttSendData>,
    /// 主题配置
    topic: Arc<TopicWrap>,
}

impl MqttClientManager {
    /// 创建新的客户端管理器
    pub fn new(
        client_ids: Vec<String>,
        send_data: Arc<MqttSendData>,
        topic: Arc<TopicWrap>,
    ) -> Self {
        Self {
            client_ids: Arc::new(client_ids),
            send_data,
            topic,
        }
    }

    pub fn get_send_data(&self) -> &Arc<MqttSendData> {
        &self.send_data
    }

    /// 获取客户端ID
    pub fn get_client_ids(&self) -> &Arc<Vec<String>> {
        &self.client_ids
    }

    /// 批量创建客户端连接
    ///
    /// 返回成功创建的客户端ID列表
    pub async fn batch_setup_clients(
        &self,
        config: &BasicConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<String>, Error> {
        let mut successful_clients = Vec::new();
        let app_state = get_app_state();
        let semaphore = Arc::new(Semaphore::new(config.get_max_connect_per_second()));

        let broker_parts: Vec<&str> = config.get_broker().split(':').collect();
        let host = broker_parts[0].trim_start_matches("tcp://");
        let port = broker_parts
            .get(1)
            .unwrap_or(&"1883")
            .parse::<u16>()
            .unwrap_or(1883);

        for client_config in config.get_clients().iter() {
            let permit = semaphore.acquire().await?;

            match self
                .setup_single_client(client_config, host, port, app_state)
                .await
            {
                Ok(_) => {
                    successful_clients.push(client_config.client_id.clone());
                }
                Err(e) => {
                    error!("设置客户端 {} 失败: {:?}", client_config.client_id, e);
                }
            }

            drop(permit);

            if successful_clients.len() % config.get_max_connect_per_second() == 0 {
                sleep(Duration::from_secs(1)).await;
            }
        }

        Ok(successful_clients)
    }

    /// 设置单个客户端
    async fn setup_single_client(
        &self,
        client_config: &MqttClientData,
        host: &str,
        port: u16,
        app_state: &AppState,
    ) -> Result<(), Error> {
        use rumqttc::{AsyncClient, MqttOptions};

        let client_id = &client_config.client_id;
        let mut mqtt_options = MqttOptions::new(client_id, host, port);

        mqtt_options.set_clean_session(true);
        mqtt_options.set_keep_alive(Duration::from_secs(20));
        mqtt_options.set_credentials(client_id, client_config.get_password());
        mqtt_options.set_request_channel_capacity(10);

        let (cli, event_loop) = AsyncClient::new(mqtt_options, 10);
        let event_loop_handle = self.spawn_event_loop(client_id.clone(), event_loop).await;

        let mut client_data = client_config.clone();
        client_data.event_loop_handle = Some(Arc::new(RwLock::new(Some(event_loop_handle))));
        client_data.set_client(Some(Arc::new(cli)));

        app_state.add_mqtt_client(client_id.clone(), client_data);

        Ok(())
    }

    /// 启动事件循环处理
    async fn spawn_event_loop(
        &self,
        client_id: String,
        mut event_loop: rumqttc::EventLoop,
    ) -> JoinHandle<()> {
        let app_state = get_app_state();

        tokio::spawn(async move {
            loop {
                if !app_state.mqtt_clients().contains_key(&client_id) {
                    break;
                }

                match event_loop.poll().await {
                    Ok(event) => {
                        Self::process_event(&event, &client_id);
                    }
                    Err(e) => {
                        if let Some(client_entry) = app_state.mqtt_clients().get_mut(&client_id) {
                            if !client_entry
                                .disconnecting
                                .load(std::sync::atomic::Ordering::SeqCst)
                            {
                                error!("MQTT事件循环错误: {:?}", e);
                            }

                            let client_entry_clone = client_entry.clone();
                            tokio::spawn(async move {
                                if let Err(e) = client_entry_clone.safe_disconnect().await {
                                    error!("断开连接失败: {:?}", e);
                                }
                            });
                            break;
                        } else {
                            error!("MQTT事件循环错误: {:?}", e);
                        }

                        if !app_state.mqtt_clients().contains_key(&client_id) {
                            break;
                        }
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }

            // 清理工作
            if let Some(mut client) = app_state.mqtt_clients().get_mut(&client_id) {
                if client.client.is_some() {
                    client.client = None;
                }
            }
        })
    }

    /// 处理MQTT事件
    fn process_event(event: &rumqttc::Event, client_id: &str) {
        use rumqttc::{Event, Packet};

        if let Event::Incoming(Packet::ConnAck(_)) = event {
            debug!("收到ConnAck事件，客户端ID: {}", client_id);

            let app_state = get_app_state();
            if let Some(mut client) = app_state.mqtt_clients().get_mut(client_id) {
                client.set_connection_state(crate::ConnectionState::Connected);
                debug!("已更新客户端连接状态为已连接: {}", client_id);
            }
        } else {
            debug!("处理其他MQTT事件: {:?}", event);
        }
    }

    /// 启动消息发送任务
    ///
    pub async fn spawn_message_tasks(
        &self,
        client_ids: Vec<String>,
        task: &Task,
        config: &BasicConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<JoinHandle<()>>, Error> {
        info!("开始发送消息...");

        let clients_per_thread = (client_ids.len() + config.thread_size - 1) / config.thread_size;
        let id_groups: Vec<Vec<String>> = client_ids
            .chunks(clients_per_thread)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(id_groups.len());

        for group in id_groups {
            let handle = self.spawn_single_message_task(group, task, config).await;
            handles.push(handle);
        }

        Ok(handles)
    }

    /// 启动单个消息发送任务
    async fn spawn_single_message_task(
        &self,
        client_ids: Vec<String>,
        task: &Task,
        config: &BasicConfig<MqttSendData, MqttClientData>,
    ) -> JoinHandle<()> {
        let send_data = Arc::clone(&self.send_data);
        let topic = Arc::clone(&self.topic);
        let counter = Arc::clone(&task.counter);
        let status = Arc::clone(&task.status);
        let send_interval = config.send_interval;
        let enable_random = config.enable_random;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(send_interval));

            loop {
                if !status.load(Ordering::SeqCst) {
                    info!("停止发送消息");
                    break;
                }

                interval.tick().await;

                for client_id in &client_ids {
                    let app_state = get_app_state();
                    let Some(client_data) = app_state.mqtt_clients().get(client_id) else {
                        continue;
                    };

                    if !client_data.is_connected() {
                        continue;
                    }

                    if let Err(e) = Self::send_single_message(
                        &client_data,
                        Arc::clone(&send_data),
                        Arc::clone(&topic),
                        Arc::clone(&counter),
                        enable_random,
                    )
                    .await
                    {
                        error!("发送消息失败 - 客户端ID: {}, 错误: {:?}", client_id, e);
                    }
                }
            }
        })
    }

    /// 发送单条消息
    async fn send_single_message(
        client_data: &MqttClientData,
        send_data: Arc<MqttSendData>,
        topic: Arc<TopicWrap>,
        counter: Arc<AtomicU32>,
        enable_random: bool,
    ) -> Result<(), Error> {
        let real_topic = match client_data.get_identify_key() {
            Some(identify_key) => topic.get_publish_real_topic_identify_key(identify_key.clone()),
            None => topic.get_publish_real_topic(Some(client_data.get_device_key())),
        };

        let mut msg_value = (*send_data).clone();
        process_fields(&mut msg_value.data, &msg_value.fields, enable_random);
        let json_msg = serde_json::to_string(&msg_value.data)?;

        let qos = topic.get_publish_qos();
        let client = client_data
            .get_client()
            .ok_or_else(|| anyhow::anyhow!("客户端未初始化"))?;

        client.publish(real_topic, qos, false, json_msg).await?;
        counter.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    /// 等待所有客户端连接完成
    pub async fn wait_for_connections(&self, client_ids: &[String]) -> bool {
        let mut futures = Vec::with_capacity(client_ids.len());

        for client_id in client_ids {
            let client_id = client_id.clone();
            futures.push(tokio::spawn(async move {
                Self::wait_single_connection(client_id).await
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
                    error!("等待连接任务失败: {:?}", e);
                    all_connected = false;
                }
            }
        }

        all_connected
    }

    /// 等待单个客户端连接
    async fn wait_single_connection(client_id: String) -> bool {
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100; // 10秒超时

        while attempts < MAX_ATTEMPTS {
            let app_state = get_app_state();
            if let Some(client_data) = app_state.mqtt_clients().get(&client_id) {
                if client_data.is_connected() {
                    return true;
                }
            }

            sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        // 连接超时，设置状态为失败
        let app_state = get_app_state();
        if let Some(mut client) = app_state.mqtt_clients().get_mut(&client_id) {
            client.set_connection_state(crate::ConnectionState::Failed);
        }

        error!("客户端 {} 连接超时", client_id);
        false
    }

    /// 获取连接统计信息
    pub fn get_connection_stats(&self) -> ConnectionStats {
        let app_state = get_app_state();
        let mut stats = ConnectionStats::default();

        for client_id in self.client_ids.iter() {
            if let Some(client_data) = app_state.mqtt_clients().get(client_id) {
                match client_data.get_connection_state() {
                    crate::ConnectionState::Connected => stats.connected += 1,
                    crate::ConnectionState::Connecting => stats.connecting += 1,
                    crate::ConnectionState::Failed => stats.failed += 1,
                }
            }
        }

        stats.total = self.client_ids.len();
        stats
    }
}

/// 连接统计信息
#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub total: usize,
    pub connected: usize,
    pub connecting: usize,
    pub failed: usize,
}

impl ConnectionStats {
    /// 获取连接成功率
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.connected as f64 / self.total as f64
        }
    }
}
