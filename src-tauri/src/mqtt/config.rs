
use anyhow::{anyhow, Result};

use crate::{
    config::BasicConfig,
    mqtt::{basic::TopicConfig, client_data::MqttClient},
    MqttClientData, MqttSendData, TopicWrap,
};

use super::hooks::init_mqtt_hooks;


/// 初始化MQTT客户端上下文
///
/// 根据配置和主题配置创建MQTT客户端实例
///
/// # 参数
/// * `config` - 基准测试配置
/// * `topic_config` - 主题配置
///
/// # 返回
/// 成功返回MQTT客户端实例，失败返回错误
pub async fn init_mqtt_context(
    config: &BasicConfig<MqttSendData, MqttClientData>,
    topic_config: TopicConfig,
) -> Result<MqttClient> {
    let mut register_topic: Option<TopicWrap> = None;
    let data_topic;
    if config.enable_register {
        if let Some(register) = topic_config.register {
            register_topic = Some(register);
        } else {
            return Err(anyhow!("没有配置注册主题"));
        }
    }
    if let Some(data) = topic_config.data {
        data_topic = data;
    } else {
        return Err(anyhow!("没有配置数据上报主题"));
    }
    let mqtt_client = MqttClient::new(
        config.get_send_data().clone(),
        config.enable_register,
        register_topic,
        data_topic,
    );
    init_mqtt_hooks(mqtt_client.clone()).await;
    Ok(mqtt_client)
}
