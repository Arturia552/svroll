use std::sync::Arc;

use crate::{
    context::get_app_state,
    mqtt::{MqttFieldStruct, TopicConfig},
    param::{BasicConfig, Protocol},
    tcp::tcp_client::{TcpClient, TcpSendData},
    MqttClientData, MqttSendData,
};
use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 连接参数配置
///
/// 包含从前端传递的所有连接和发送配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectParam {
    /// 要发送的数据内容（以JSON或十六进制格式）
    #[serde(rename = "sendData")]
    pub send_data: String,
    /// 客户端列表，表示为JSON值数组
    #[serde(rename = "clients")]
    pub clients: Vec<Value>,
    /// 使用的协议类型
    pub protocol: Protocol,
    /// 线程数量
    #[serde(rename = "threadSize")]
    pub thread_size: usize,
    /// 是否启用随机值生成
    #[serde(rename = "enableRandom")]
    pub enable_random: bool,
    /// 服务器地址
    #[serde(rename = "broker")]
    pub broker: String,
    /// 每秒最大连接数
    #[serde(rename = "maxConnectPerSecond")]
    pub max_connect_per_second: usize,
    /// 发送间隔(秒)
    #[serde(rename = "sendInterval")]
    pub send_interval: u64,
    /// MQTT字段结构定义
    #[serde(rename = "fieldStruct")]
    pub field_struct: Vec<MqttFieldStruct>,
    /// 主题配置
    #[serde(rename = "topicConfig")]
    pub topic_config: Option<TopicConfig>,
}

impl ConnectParam {
    /// 转换为MQTT配置
    ///
    /// 将通用连接参数转换为MQTT特定的基准测试配置
    ///
    /// # 返回
    /// 成功返回MQTT配置，失败返回错误
    pub fn into_config(&self) -> Result<BasicConfig<MqttSendData, MqttClientData>> {
        let data: Value =
            serde_json::from_str(self.send_data.as_str()).with_context(|| "发送数据格式错误")?;
        let send_data = MqttSendData {
            data,
            fields: self.field_struct.clone(),
        };

        let mut clients = vec![];
        for client in self.clients.iter() {
            let client_data: MqttClientData =
                serde_json::from_value(client.clone()).with_context(|| "客户端数据格式错误")?;
            let app_state = get_app_state();
            app_state
                .mqtt_clients()
                .insert(client_data.get_client_id().to_string(), client_data.clone());
            clients.push(client_data);
        }
        Ok(BasicConfig::new(
            send_data,
            clients,
            Protocol::Mqtt,
            self.thread_size,
            self.enable_random,
            self.broker.clone(),
            self.max_connect_per_second,
            self.send_interval,
        ))
    }

    pub fn set_send_data(&mut self, send_data: String) {
        self.send_data = send_data;
    }

    /// 转换为TCP配置
    ///
    /// 将通用连接参数转换为TCP特定的基准测试配置
    ///
    /// # 返回
    /// 成功返回TCP配置，失败返回错误
    pub fn into_tcp_config(&self) -> Result<BasicConfig<TcpSendData, TcpClient>> {
        let send_data = hex::decode(&self.send_data).with_context(|| "发送数据格式错误")?;

        let mut clients = vec![];
        for client in self.clients.iter() {
            let client_data: TcpClient =
                serde_json::from_value(client.clone()).with_context(|| "客户端数据格式错误")?;
            let app_state = get_app_state();
            app_state.tcp_clients().insert(
                client_data.get_mac().to_string(),
                (client_data.clone(), None),
            );
            clients.push(client_data);
        }

        Ok(BasicConfig::new(
            TcpSendData {
                data: Arc::new(send_data),
            },
            clients,
            Protocol::Tcp,
            self.thread_size,
            self.enable_random,
            self.broker.clone(),
            self.max_connect_per_second,
            self.send_interval,
        ))
    }
}
