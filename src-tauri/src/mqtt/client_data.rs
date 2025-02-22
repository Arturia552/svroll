use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use paho_mqtt::{
    disconnect_options, AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder,
    DisconnectOptionsBuilder, Message,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::{
    sync::Semaphore,
    time::{sleep, Instant},
};
use tracing::info;

use crate::{
    benchmark_param::BenchmarkConfig, model::tauri_com::Task, MqttSendData, TopicWrap,
    MQTT_CLIENT_CONTEXT,
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

    pub fn set_register_topic(&mut self, topic: Option<TopicWrap>) {
        self.register_topic = Arc::new(topic);
    }

    pub fn get_register_topic(&self) -> Option<&TopicWrap> {
        self.register_topic.as_ref().as_ref()
    }

    pub fn set_data_topic(&mut self, topic: TopicWrap) {
        self.data_topic = Arc::new(topic);
    }

    pub fn get_data_topic(&self) -> &TopicWrap {
        &self.data_topic
    }

    fn get_real_topic_mac(topic: &str) -> (String, String) {
        let topic = topic.to_string();
        let mut topic = topic.split('/').collect::<Vec<&str>>();
        let mac = topic.remove(2);
        (topic.join("/"), mac.to_string())
    }

    pub fn on_message_callback(&self, _: &AsyncClient, msg: Option<Message>) {
        if let Some(msg) = msg {
            let topic = msg.topic(); // 获取消息主题

            // 检查主题是否以"/sub"开头
            if topic.starts_with("/sub") {
                // 获取真实的主题和MAC地址
                let (real_topic, mac) = Self::get_real_topic_mac(topic);
                let data = msg.payload(); // 获取消息负载
                                          // 尝试将负载解析为JSON
                if let Ok(data) = serde_json::from_slice::<serde_json::Value>(data) {
                    if self.get_enable_register() {
                        let register_topic = self.get_register_topic();
                        let reg_sub_topic_wrap = register_topic.unwrap();
                        let reg_sub_topic = reg_sub_topic_wrap.get_subscribe_topic().unwrap();
                        let extra_key = reg_sub_topic_wrap.subscribe.clone().unwrap().extra_key.unwrap();
                        // 检查真实主题是否为注册包回复
                        if real_topic == reg_sub_topic {
                            // 检查JSON对象中是否存在"device_key"
                            if let Some(device_key) = data.get(extra_key) {
                                // 将device_key转换为字符串
                                if let Some(device_key_str) = device_key.as_str() {
                                    // 更新CLIENT_CONTEXT中的device_key
                                    MQTT_CLIENT_CONTEXT.entry(mac.to_string()).and_modify(|v| {
                                        v.set_device_key(device_key_str.to_string());
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Client<MqttSendData, MqttClientData> for MqttClient {
    type Item = AsyncClient;

    async fn setup_clients(
        &self,
        config: &BenchmarkConfig<MqttSendData, MqttClientData>,
    ) -> Result<Vec<AsyncClient>, Box<dyn std::error::Error>> {
        let mut clients = vec![];

        let broker = config.get_broker();
        let semaphore = Arc::new(Semaphore::new(config.get_max_connect_per_second()));

        for client in config.get_clients() {
            MQTT_CLIENT_CONTEXT.insert(client.get_client_id().to_string(), client.clone());
            let create_opts = CreateOptionsBuilder::new()
                .server_uri(broker)
                .client_id(&client.client_id)
                .finalize();
            let mut cli: AsyncClient = AsyncClient::new(create_opts)?;

            let conn_opts = ConnectOptionsBuilder::new_v5()
                .clean_start(true)
                .automatic_reconnect(Duration::from_secs(2), Duration::from_secs(2))
                .keep_alive_interval(Duration::from_secs(20))
                .user_name(&client.client_id)
                .password(client.get_password())
                .finalize();

            let mqtt_client = self.clone();
            cli.set_message_callback(move |client, message| {
                Self::on_message_callback(&mqtt_client, client, message);
            });

            let mqtt_client = self.clone();
            clients.push(cli.clone());

            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let permit = semaphore.acquire().await.unwrap();

                let start = Instant::now();
                match cli.connect(conn_opts).await {
                    Ok(_) => {
                        mqtt_client.on_connect_success(&mut cli).await;
                    }
                    Err(_) => todo!(),
                }

                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(1) {
                    tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
                }

                drop(permit);
            });
        }

        Ok(clients)
    }

    async fn on_connect_success(&self, cli: &mut Self::Item) {
        // 注册包机制启用判断
        if self.get_enable_register() {
            match self.get_register_topic() {
                Some(topic) => {
                    if let Some(extra_key) = &topic.publish.extra_key {
                        // 创建订阅主题并订阅
                        let sub_topic = self.get_register_topic().unwrap();

                        if sub_topic.is_exist_subscribe() {
                            let sub_topic_str =
                                sub_topic.get_subscribe_real_topic(Some(cli.client_id().as_str()));
                            let _ = cli.subscribe(sub_topic_str, sub_topic.get_subscribe_qos());
                        }

                        let pub_topic = self.get_register_topic().unwrap();
                        let pub_topic_str = pub_topic.get_publish_topic();

                        let register_json_str =
                            format!(r#"{{"{}": "{}"}}"#, extra_key, cli.client_id());
                        let register_msg = Message::new(
                            pub_topic_str,
                            register_json_str,
                            pub_topic.get_publish_qos(),
                        );
                        cli.publish(register_msg);
                    } else {
                        let disconnect_options = DisconnectOptionsBuilder::new().finalize();
                        cli.disconnect(disconnect_options);
                    }
                }
                None => {
                    // 断开连接
                    let disconnect_options = DisconnectOptionsBuilder::new().finalize();
                    cli.disconnect(disconnect_options);
                    println!("没有配置注册主题");
                }
            }
        }
    }

    async fn spawn_message(
        &self,
        clients: Vec<Self::Item>,
        task: &Task,
        config: &BenchmarkConfig<MqttSendData, MqttClientData>,
    ) {
        info!("开始发送消息...");
        info!(?task);
        // 确定每个线程处理的客户端数量
        let startup_thread_size = clients.len() / config.thread_size
            + if clients.len() % config.thread_size != 0 {
                1
            } else {
                0
            };
        let clients_group = clients.chunks(startup_thread_size);

        for group in clients_group {
            let mut group = group.to_vec();
            let send_data = Arc::clone(&self.send_data);
            let counter: Arc<AtomicU32> = task.counter.clone();
            let status: Arc<AtomicBool> = task.status.clone();
            let mqtt_client = self.clone();
            let topic = Arc::clone(&self.data_topic);
            let send_interval = config.send_interval;
            let enable_register = config.enable_register;
            let enable_random = config.enable_random;

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(send_interval));
                loop {
                    if !status.load(Ordering::SeqCst) {
                        info!("停止发送消息");
                        break;
                    }

                    // 等待指定的间隔时间再进行下一次发送
                    interval.tick().await;
                    // 遍历每个组中的客户端
                    for cli in group.iter_mut() {
                        if !cli.is_connected() {
                            continue;
                        }
                        let client_id = cli.client_id().to_string();
                        if let Some(client_data) = MQTT_CLIENT_CONTEXT.get(&client_id) {
                            let device_key = client_data.get_device_key();
                            if device_key.is_empty() && enable_register {
                                mqtt_client.on_connect_success(cli).await;
                                continue;
                            }
                            let real_topic =
                                topic.get_publish_real_topic(Some(client_data.get_device_key()));

                            let mut msg_value = (*send_data).clone();
                            msg_value.process_fields(enable_random);
                            let json_msg = match serde_json::to_string(&msg_value.data) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    eprintln!("序列化JSON失败: {}", e);
                                    return;
                                }
                            };
                            info!(?json_msg);
                            let payload: Message =
                                Message::new(real_topic, json_msg, topic.get_publish_qos());
                            counter.fetch_add(1, Ordering::SeqCst);
                            let _ = cli.publish(payload);
                        }
                    }
                }
            });
        }
    }

    async fn wait_for_connections(&self, clients: &mut [AsyncClient]) {
        for ele in clients {
            while !ele.is_connected() {
                sleep(Duration::from_secs(1)).await;
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

    pub fn set_device_key(&mut self, device_key: String) {
        self.device_key = device_key;
    }
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
