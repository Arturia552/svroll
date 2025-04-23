use std::borrow::Cow;

use rumqttc::QoS;
use serde::{Deserialize, Serialize};

/// MQTT主题信息
///
/// 定义MQTT主题的属性和行为，包括主题路径、QoS等级等
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicInfo {
    /// 主题路径中关键标识的索引位置
    #[serde(rename = "keyIndex")]
    pub key_index: Option<usize>,
    /// 主题路径
    pub topic: String,
    /// 额外的关键字，用于注册和识别
    #[serde(rename = "extraKey")]
    pub extra_key: Option<String>,
    /// 服务质量等级 (0-最多一次, 1-至少一次, 2-恰好一次)
    #[serde(default = "default_qos")]
    pub qos: i32,
}

/// 默认QoS设置
///
/// 返回默认的QoS级别(0)
pub fn default_qos() -> i32 {
    0
}

impl TopicInfo {
    /// 获取主题路径
    pub fn get_topic(&self) -> &str {
        &self.topic
    }

    /// 获取QoS级别
    pub fn get_qos(&self) -> i32 {
        self.qos
    }
}

/// 主题包装器
///
/// 包含发布和订阅主题的配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicWrap {
    /// 发布主题配置
    pub publish: TopicInfo,
    /// 可选的订阅主题配置
    #[serde(default)]
    pub subscribe: Option<TopicInfo>,
}

impl TopicWrap {
    /// 检查是否存在订阅主题
    pub fn is_exist_subscribe(&self) -> bool {
        self.subscribe.is_some()
    }

    /// 获取发布主题路径
    pub fn get_publish_topic(&self) -> &str {
        &self.publish.get_topic()
    }

    /// 获取订阅主题路径
    ///
    /// # 返回
    /// 如果配置了订阅主题，返回Some(主题路径)，否则返回None
    pub fn get_subscribe_topic(&self) -> Option<&str> {
        self.subscribe
            .as_ref()
            .map(|sub_topic| sub_topic.get_topic())
    }

    /// 获取发布主题的QoS级别
    ///
    /// 将整数QoS转换为MQTT QoS枚举
    pub fn get_publish_qos(&self) -> QoS {
        match self.publish.get_qos() {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        }
    }

    /// 获取订阅主题的QoS级别
    ///
    /// 将整数QoS转换为MQTT QoS枚举
    pub fn get_subscribe_qos(&self) -> QoS {
        match self.subscribe.as_ref().unwrap().qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtMostOnce,
        }
    }

    /// 获取实际的发布主题路径
    ///
    /// 根据主题模板和关键值构建完整主题路径
    ///
    /// # 参数
    /// * `key_value` - 可选的关键值，用于替换主题路径中的占位符
    pub fn get_publish_real_topic<'a>(&'a self, key_value: Option<&str>) -> Cow<'a, str> {
        wrap_real_topic(&self.publish, key_value)
    }

    pub fn get_pushlish_real_topic_identify_key<'a>(&'a self, identify_key: String) -> Cow<'a,str> {
        let topic = &self.publish;
        let key_index = topic.key_index;
        if key_index.is_none() || identify_key.trim().is_empty() || key_index == Some(0) {
            return Cow::Borrowed(&topic.topic);
        }else {
            let key_index = key_index.unwrap();
            let parts: Vec<&str> = topic.topic.split('/').collect();

            if key_index < parts.len() {
                let mut new_topic_parts = parts[..key_index].to_vec();
                new_topic_parts.push(&identify_key);
                new_topic_parts.extend(&parts[key_index..]);

                let new_topic = new_topic_parts.join("/");
                return Cow::Owned(new_topic);
            }
            return Cow::Borrowed(&topic.topic);
        }
        
    }

    /// 获取实际的订阅主题路径
    ///
    /// 根据主题模板和关键值构建完整主题路径
    ///
    /// # 参数
    /// * `key_value` - 可选的关键值，用于替换主题路径中的占位符
    pub fn get_subscribe_real_topic<'a>(&'a self, key_value: Option<&str>) -> Cow<'a, str> {
        if let Some(topic) = &self.subscribe {
            wrap_real_topic(topic, key_value)
        } else {
            Cow::Borrowed("")
        }
    }
}

/// 构建实际的主题路径
///
/// 根据主题模板和关键值拼接完整的主题路径
///
/// # 参数
/// * `topic` - 主题信息模板
/// * `key_value` - 可选的关键值，用于替换主题路径中的占位符
///
/// # 返回
/// 返回构建好的主题路径
pub fn wrap_real_topic<'a>(topic: &'a TopicInfo, key_value: Option<&str>) -> Cow<'a, str> {
    if topic.key_index.unwrap_or(0) == 0
        || key_value.is_none()
        || key_value.is_some_and(|val| val.is_empty())
    {
        return Cow::Borrowed(&topic.topic);
    } else {
        let key_index = topic.key_index.unwrap_or(0);
        let parts: Vec<&str> = topic.topic.split('/').collect();

        if key_index < parts.len() {
            let mut new_topic_parts = parts[..key_index].to_vec();
            if let Some(value) = key_value {
                new_topic_parts.push(value);
            }
            new_topic_parts.extend(&parts[key_index..]);

            let new_topic = new_topic_parts.join("/");
            Cow::Owned(new_topic)
        } else {
            Cow::Borrowed(&topic.topic)
        }
    }
}

/// MQTT主题配置
///
/// 包含注册和数据传输的主题配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicConfig {
    /// 设备注册主题配置
    pub register: Option<TopicWrap>,
    /// 数据传输主题配置
    pub data: Option<TopicWrap>,
}

impl TopicConfig {
    /// 获取注册主题路径
    ///
    /// # 返回
    /// 如果配置了注册主题，返回Some(主题路径)，否则返回None
    pub fn get_register_topic(&self) -> Option<&str> {
        self.register
            .as_ref()
            .map(|topic| topic.get_publish_topic())
    }
}

/// TCP配置结构体
///
/// 用于存储TCP连接的配置参数
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TcpConfig {}

impl Default for TopicConfig {
    fn default() -> Self {
        TopicConfig {
            register: Some(TopicWrap {
                publish: TopicInfo {
                    key_index: None,
                    extra_key: None,
                    topic: "/pub/register".to_string(),
                    qos: default_qos(),
                },
                subscribe: Some(TopicInfo {
                    key_index: Some(2),
                    extra_key: None,
                    topic: "/sub/register/ack".to_string(),
                    qos: default_qos(),
                }),
            }),
            data: Some(TopicWrap {
                publish: TopicInfo {
                    key_index: Some(2),
                    extra_key: None,
                    topic: "/pub/long_freq/data".to_string(),
                    qos: default_qos(),
                },
                subscribe: None,
            }),
        }
    }
}
