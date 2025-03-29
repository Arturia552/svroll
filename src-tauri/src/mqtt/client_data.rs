use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
    vec,
};

use anyhow::{Error, Result};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{Mutex, Semaphore},
    task::JoinHandle,
    time::sleep,
};
use tracing::{error, info};

use crate::{
    benchmark_param::BenchmarkConfig, context::get_app_state, model::tauri_com::Task, mqtt::device_data::process_fields, ConnectionState, MqttSendData, TopicWrap
};

use super::Client;

#[derive(Clone)]
pub struct MqttClient {
    pub send_data: Arc<MqttSendData>,
    pub enable_register: bool,
    pub register_topic: Arc<Option<TopicWrap>>,
    pub data_topic: Arc<TopicWrap>,
}

impl MqttClient {
    pub fn new(
        send_data: MqttSendData,
        enable_register: bool,
        register_topic: Option<TopicWrap>,
        data_topic: TopicWrap,
    ) -> Self {
        MqttClient {
            send_data: Arc::new(send_data),
            enable_register,
            register_topic: Arc::new(register_topic),
            data_topic: Arc::new(data_topic),
        }
    }

    pub fn get_send_data(&self) -> &MqttSendData {
        &self.send_data
    }

    pub fn get_enable_register(&self) -> bool {
        self.enable_register
    }

    pub fn set_enable_register(&mut self, enable: bool) {
        self.enable_register = enable;
    }

    pub fn get_register_topic(&self) -> Option<&TopicWrap> {
        self.register_topic.as_ref().as_ref()
    }

    fn parse_topic_mac(topic: &str, key_index: usize) -> (String, String) {
        let topic = topic.to_string();
        let mut topic = topic.split('/').collect::<Vec<&str>>();
        let mac = topic.remove(key_index);
        (topic.join("/"), mac.to_string())
    }

    fn on_message_callback(&self, topic: &str, payload: &[u8]) {
        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(payload) {
            if self.get_enable_register() {
                let Some(register_topic) = self.get_register_topic() else {
                    return;
                };
                let Some(reg_subscribe) = &register_topic.subscribe else {
                    return;
                };

                let (real_topic, mac) =
                    Self::parse_topic_mac(topic, reg_subscribe.key_index.unwrap());

                let Some(reg_sub_topic) = register_topic.get_subscribe_topic() else {
                    return;
                };
                let Some(extra_key) = &reg_subscribe.extra_key else {
                    return;
                };

                if real_topic != reg_sub_topic {
                    return;
                }

                if let Some(device_key) = data.get(extra_key) {
                    if let Some(device_key_str) = device_key.as_str() {
                        get_app_state().mqtt_clients().entry(mac.to_string()).and_modify(|v| {
                            v.set_device_key(device_key_str.to_string());
                        });
                    }
                }
            }
        }
    }

    // 处理事件循环
    async fn handle_event_loop(
        client_id: String,
        mut event_loop: EventLoop,
        self_clone: Arc<MqttClient>,
    ) -> JoinHandle<()> {
        let app_state = get_app_state();
        // 返回JoinHandle以便后续可以取消
        tokio::spawn(async move {
            loop {
                // 每次循环前检查客户端是否还存在
                if !app_state.mqtt_clients().contains_key(&client_id) {
                    break;
                }

                match event_loop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        self_clone.on_message_callback(&publish.topic, &publish.payload);
                    }
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        if let Some(mut client) = app_state.mqtt_clients().get_mut(&client_id) {
                            match self_clone.on_connect_success(&mut client).await {
                                Ok(_) => {
                                    client.set_connection_state(ConnectionState::Connected);
                                }
                                Err(e) => {
                                    error!("连接初始化失败: {:?}", e);
                                    client.set_connection_state(ConnectionState::Failed);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(mut client_entry) = app_state.mqtt_clients().get_mut(&client_id) {
                            client_entry.set_connection_state(ConnectionState::Failed);

                            if !client_entry.disconnecting.load(Ordering::SeqCst) {
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
                    _ => {}
                }
            }
            if let Some(mut client) = app_state.mqtt_clients().get_mut(&client_id) {
                client.set_connection_state(ConnectionState::Failed);
                if client.client.is_some() {
                    client.client = None;
                }
            }
        })
    }
}

impl Client<MqttSendData, MqttClientData> for MqttClient {
    type Item = MqttClientData;

    async fn setup_clients(
        &self,
        config: &BenchmarkConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<MqttClientData>, Error> {
        let mut clients = vec![];

        let app_state = get_app_state();
        let self_arc = Arc::new(self.clone());
        let broker = config.get_broker();
        let semaphore = Arc::new(Semaphore::new(config.get_max_connect_per_second()));

        // 解析broker地址和端口
        let broker_parts: Vec<&str> = broker.split(':').collect();
        let host = broker_parts[0].trim_start_matches("tcp://");
        let port = broker_parts
            .get(1)
            .unwrap_or(&"1883")
            .parse::<u16>()
            .unwrap_or(1883);

        for client_ref in config.get_clients() {
            let permit = semaphore.acquire().await?;

            let mut client = client_ref.clone();
            let mut mqtt_options = MqttOptions::new(&client.client_id, host, port);

            mqtt_options.set_clean_session(true);
            mqtt_options.set_keep_alive(Duration::from_secs(20));
            mqtt_options.set_credentials(&client.client_id, client.get_password());
            mqtt_options.set_request_channel_capacity(10);

            // 创建客户端和事件循环
            let (cli, event_loop) = AsyncClient::new(mqtt_options, 10);
            // 启动事件循环处理
            let client_id = client.client_id.clone();
            let event_loop_handle: JoinHandle<()> =
                Self::handle_event_loop(client_id.clone(), event_loop, Arc::clone(&self_arc)).await;
            client.event_loop_handle = Some(Arc::new(Mutex::new(Some(event_loop_handle))));
            client.set_client(Some(cli.clone()));
           
            app_state.add_mqtt_client(client.get_client_id().to_string(), client.clone());
            clients.push(client.clone());

            drop(permit);

            // 添加小延迟确保连接速率控制更平滑
            if clients.len() % config.get_max_connect_per_second() == 0 {
                sleep(Duration::from_secs(1)).await;
            }
        }

        Ok(clients)
    }

    async fn on_connect_success(&self, cli: &mut MqttClientData) -> Result<(), Error> {
        let client = cli
            .get_client()
            .ok_or_else(|| Error::msg("客户端未初始化"))?;

        // 注册包机制启用判断
        if self.get_enable_register() {
            let register_topic = match self.get_register_topic() {
                Some(topic) => topic,
                None => {
                    // 断开连接并返回错误
                    if let Err(e) = client.disconnect().await {
                        error!("断开连接失败: {:?}", e);
                    }
                    return Err(Error::msg("没有配置注册主题"));
                }
            };

            // 检查 extra_key
            let extra_key = match &register_topic.publish.extra_key {
                Some(key) => key,
                None => {
                    // 断开连接并返回错误
                    if let Err(e) = client.disconnect().await {
                        error!("断开连接失败: {:?}", e);
                    }
                    return Err(Error::msg("注册主题配置错误"));
                }
            };

            // 订阅主题处理
            if register_topic.is_exist_subscribe() {
                let sub_topic_str = register_topic.get_subscribe_real_topic(Some(&cli.client_id));
                let qos = register_topic.get_subscribe_qos();

                if let Err(e) = client.subscribe(sub_topic_str, qos).await {
                    error!("订阅主题失败: {:?}", e);
                    return Err(Error::msg(format!("订阅主题失败: {:?}", e)));
                }
            }

            // 发布注册消息
            let pub_topic_str = register_topic.get_publish_topic();
            let register_json_str = format!(r#"{{"{}": "{}"}}"#, extra_key, cli.client_id);
            let qos = register_topic.get_publish_qos();

            if let Err(e) = client
                .publish(pub_topic_str, qos, false, register_json_str)
                .await
            {
                error!("发布注册消息失败: {:?}", e);
                return Err(Error::msg(format!("发布注册消息失败: {:?}", e)));
            }
        }
        Ok(())
    }

    async fn spawn_message(
        &self,
        clients: Vec<Self::Item>,
        task: &Task,
        config: &BenchmarkConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<JoinHandle<()>>, Error> {
        info!("开始发送消息...");
        // 确定每个线程处理的客户端数量
        let clients_per_thread = (clients.len() + config.thread_size - 1) / config.thread_size;
        let clients_group = clients.chunks(clients_per_thread);
        let mut handles: Vec<JoinHandle<()>> = vec![];
        let app_state = get_app_state();
        let self_arc = Arc::new(self.clone());

        for group in clients_group {
            let group = group.to_vec();
            let send_data = Arc::clone(&self.send_data);
            let counter: Arc<AtomicU32> = task.counter.clone();
            let status: Arc<AtomicBool> = task.status.clone();
            let mqtt_client = Arc::clone(&self_arc);
            let topic = Arc::clone(&self.data_topic);
            let send_interval = config.send_interval;
            let enable_register = config.enable_register;
            let enable_random = config.enable_random;

            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(send_interval));
                loop {
                    if !status.load(Ordering::SeqCst) {
                        info!("停止发送消息");
                        break;
                    }

                    // 等待指定的间隔时间再进行下一次发送
                    interval.tick().await;
                    // 遍历每个组中的客户端
                    for cli in &group {
                        let client_id = cli.get_client_id().to_string();
                        let Some(client_data) = app_state.mqtt_clients().get(&client_id) else {
                            continue;
                        };

                        if !client_data.is_connected() {
                            continue;
                        }

                        let device_key = client_data.get_device_key();
                        if device_key.is_empty() && enable_register {
                            let mut cli_clone = cli.clone();
                            let _ = mqtt_client.on_connect_success(&mut cli_clone).await;
                            continue;
                        }
                        let real_topic =
                            topic.get_publish_real_topic(Some(client_data.get_device_key()));
                        let mut msg_value = (*send_data).clone();
                        process_fields(&mut msg_value.data, &msg_value.fields, enable_random);
                        let json_msg = match serde_json::to_string(&msg_value.data) {
                            Ok(msg) => msg,
                            Err(e) => {
                                eprintln!("序列化JSON失败: {}", e);
                                return;
                            }
                        };

                        let qos = topic.get_publish_qos();

                        let client = match cli.get_client() {
                            Some(c) => c,
                            None => {
                                error!("客户端未初始化");
                                continue;
                            }
                        };

                        if let Err(e) = client.publish(real_topic, qos, false, json_msg).await {
                            error!("发布消息失败: {:?}", e);
                            continue;
                        }

                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            handles.push(handle);
        }
        anyhow::Result::Ok(handles)
    }

    async fn wait_for_connections(&self, clients: &mut [MqttClientData]) {
        let mut futures = Vec::with_capacity(clients.len());
        let app_state = get_app_state();
        for client in clients.iter() {
            let client_id = client.get_client_id().to_string();
            futures.push(tokio::spawn(async move {
                let mut attempts = 0;
                const MAX_ATTEMPTS: usize = 100; // 10秒超时

                while attempts < MAX_ATTEMPTS {
                    if let Some(client_data) = app_state.mqtt_clients().get(&client_id) {
                        if client_data.is_connected() {
                            break;
                        }
                    }
                    sleep(Duration::from_millis(100)).await;
                    attempts += 1;
                }

                if attempts >= MAX_ATTEMPTS {
                    error!("客户端 {} 连接超时", client_id);
                }
            }));
        }

        for future in futures {
            // 更好的错误处理
            if let Err(e) = future.await {
                error!("等待连接任务失败: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct MqttClientData {
    #[serde(rename = "clientId")]
    pub client_id: String,
    pub username: String,
    pub password: String,
    #[serde(skip)]
    pub device_key: String,
    #[serde(default)]
    #[serde(rename = "connectionState")]
    pub connection_state: ConnectionState,
    #[serde(skip)]
    pub client: Option<AsyncClient>,
    #[serde(skip)]
    pub event_loop_handle: Option<Arc<Mutex<Option<JoinHandle<()>>>>>,
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

    pub fn set_client(&mut self, client: Option<AsyncClient>) {
        self.client = client;
    }

    pub fn get_client(&self) -> Option<AsyncClient> {
        self.client.clone()
    }

    pub async fn safe_disconnect(&self) -> Result<(), Error> {
        // 仅当未在断开连接状态时执行
        if !self.disconnecting.swap(true, Ordering::SeqCst) {
            if let Some(client) = &self.client {
                client.disconnect().await?;
            }
        }
        Ok(())
    }
}
