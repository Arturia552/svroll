use crate::{
    benchmark_param::{BenchmarkConfig, Protocol},
    mqtt::{MqttFieldStruct, TopicConfig},
    tcp::tcp_client::TcpClientData,
    MqttClientData, MqttSendData,
};
use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectParam {
    #[serde(rename = "sendData")]
    pub send_data: String,
    #[serde(rename = "clients")]
    pub clients: Vec<MqttClientData>,
    pub protocol: Protocol,
    #[serde(rename = "threadSize")]
    pub thread_size: usize,
    #[serde(rename = "enableRegister")]
    pub enable_register: bool,
    #[serde(rename = "enableRandom")]
    pub enable_random: bool,
    #[serde(rename = "broker")]
    pub broker: String,
    #[serde(rename = "maxConnectPerSecond")]
    pub max_connect_per_second: usize,
    #[serde(rename = "sendInterval")]
    pub send_interval: u64,
    #[serde(rename = "fieldStruct")]
    pub field_struct: Vec<MqttFieldStruct>,
    #[serde(rename = "topicConfig")]
    pub topic_config: Option<TopicConfig>,
}

impl ConnectParam {
    pub async fn into_config(&self) -> Result<BenchmarkConfig<MqttSendData, MqttClientData>> {
        let data: Value =
            serde_json::from_str(self.send_data.as_str()).with_context(|| "发送数据格式错误")?;
        let send_data = MqttSendData {
            data,
            fields: self.field_struct.clone(),
        };
        Ok(BenchmarkConfig {
            send_data,
            protocol_type: Protocol::Mqtt,
            clients: self.clients.clone(),
            thread_size: self.thread_size,
            enable_register: self.enable_register,
            enable_random: self.enable_random,
            broker: self.broker.clone(),
            max_connect_per_second: self.max_connect_per_second,
            send_interval: self.send_interval,
        })
    }

    pub async fn set_send_data(&mut self, send_data: String) {
        self.send_data = send_data;
    }

    pub async fn into_tcp_config(&self) -> Result<BenchmarkConfig<Vec<u8>, TcpClientData>> {
        let client_file_content = " ".to_string();
        let client_data: Vec<TcpClientData> = serde_json::from_str(client_file_content.as_str())
            .with_context(|| "客户端文件格式错误")?;

        let send_data = hex::decode(&self.send_data).with_context(|| "发送数据格式错误")?;

        Ok(BenchmarkConfig::new(
            send_data,
            client_data,
            Protocol::Tcp,
            self.thread_size,
            self.enable_register,
            self.enable_random,
            self.broker.clone(),
            self.max_connect_per_second,
            self.send_interval,
        ))
    }
}
