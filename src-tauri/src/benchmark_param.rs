use std::{fmt::Debug, fs::File};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::fs::{self};

use crate::{
    mqtt::{basic::TopicConfig, client_data::MqttClient},
    MqttClientData, MqttSendData, TopicWrap,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Protocol {
    Mqtt,
    Tcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Flag {
    True,
    False,
}
/// 客户端手动输入的配置信息
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkConfig<T, C> {
    /// 设置需发送的数据内容
    pub send_data: T,

    /// 使用的数据传输协议，默认为mqtt
    pub protocol_type: Protocol,

    pub clients: Vec<C>,

    /// 设置启动协程数量,默认为200
    pub thread_size: usize,

    /// 设置是否启用注册包机制
    pub enable_register: bool,

    /// 是否启用随机值
    pub enable_random: bool,

    /// 设置mqtt broker地址,默认为mqtt://localhost:1883
    pub broker: String,

    /// 每秒最多启动连接数
    pub max_connect_per_second: usize,

    /// 设置发送间隔,默认为1秒
    pub send_interval: u64,
}

impl<T, C> BenchmarkConfig<T, C>
where
    T: DeserializeOwned + Debug,
    C: DeserializeOwned + Debug,
{
    pub fn new(
        send_data: T,
        clients: Vec<C>,
        protocol_type: Protocol,
        thread_size: usize,
        enable_register: bool,
        enable_random: bool,
        broker: String,
        max_connect_per_second: usize,
        send_interval: u64,
    ) -> Self {
        Self {
            send_data,
            protocol_type,
            clients,
            thread_size,
            enable_register,
            enable_random,
            broker,
            max_connect_per_second,
            send_interval,
        }
    }

    pub async fn validate(&self) -> Result<(), String> {
        if self.thread_size == 0 {
            return Err("线程数量不能为0".into());
        }
        if self.max_connect_per_second == 0 {
            return Err("每秒最大连接数不能为0".into());
        }
        if self.send_interval == 0 {
            return Err("发送间隔不能为0".into());
        }
        if self.clients.is_empty() {
            return Err("客户端配置不能为空".into());
        }
        if self.broker.is_empty() {
            return Err("broker地址不能为空".into());
        }
        Ok(())
    }

    pub fn set_send_data(&mut self, data: T) {
        self.send_data = data;
    }

    pub fn get_send_data(&self) -> &T {
        &self.send_data
    }

    pub fn get_broker(&self) -> &str {
        &self.broker
    }

    pub fn get_clients(&self) -> &Vec<C> {
        &self.clients
    }

    pub fn get_max_connect_per_second(&self) -> usize {
        self.max_connect_per_second
    }

    pub fn set_send_interval(&mut self, send_interval: u64) {
        self.send_interval = send_interval;
    }
}
pub async fn load_send_data_from_json_file<T>(file_path: &str) -> Result<T>
where
    T: DeserializeOwned + Debug,
{
    let contents = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("Failed to read the file: {}", file_path))?;

    // 解析 JSON 内容，并在出错时添加上下文信息
    let msg: T = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;

    Ok(msg)
}

pub async fn read_from_csv_into_struct<C>(file_path: &str) -> Result<Vec<C>>
where
    C: DeserializeOwned + Debug,
{
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b',').from_reader(file);
    let mut csv_content_vec: Vec<C> = vec![];
    for result in rdr.deserialize::<C>() {
        let record = result?;
        csv_content_vec.push(record);
    }
    Ok(csv_content_vec)
}

pub fn init_mqtt_context(
    config: &BenchmarkConfig<MqttSendData, MqttClientData>,
    topic_config: TopicConfig,
) -> Result<MqttClient, Box<dyn std::error::Error>> {
    let mut register_topic: Option<TopicWrap> = None;
    let data_topic;
    if config.enable_register {
        if let Some(register) = topic_config.register {
            register_topic = Some(register);
        } else {
            return Err("没有配置注册主题".into());
        }
    }
    if let Some(data) = topic_config.data {
        data_topic = data;
    } else {
        return Err("没有配置数据上报主题".into());
    }
    let mqtt_client = MqttClient::new(
        config.get_send_data().clone(),
        config.enable_register,
        register_topic,
        data_topic,
    );

    Ok(mqtt_client)
}
