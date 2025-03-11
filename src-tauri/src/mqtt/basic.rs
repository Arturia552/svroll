use std::borrow::Cow;

use rumqttc::QoS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicInfo {
    #[serde(rename = "keyIndex")]
    pub key_index: Option<usize>,
    pub topic: String,
    #[serde(rename = "extraKey")]
    pub extra_key: Option<String>,
    #[serde(default = "default_qos")]
    pub qos: i32,
}

pub fn default_qos() -> i32 {
    0
}

impl TopicInfo {
    pub fn get_topic(&self) -> &str {
        &self.topic
    }

    pub fn get_qos(&self) -> i32 {
        self.qos
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicWrap {
    pub publish: TopicInfo,
    #[serde(default)]
    pub subscribe: Option<TopicInfo>,
}

impl TopicWrap {
    pub fn is_exist_subscribe(&self) -> bool {
        self.subscribe.is_some()
    }

    pub fn get_publish_topic(&self) -> &str {
        &self.publish.get_topic()
    }

    pub fn get_subscribe_topic(&self) -> Option<&str> {
        self.subscribe
            .as_ref()
            .map(|sub_topic| sub_topic.get_topic())
    }

    pub fn get_publish_qos(&self) -> QoS {
       match self.publish.get_qos() {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
       }
    }

    pub fn get_subscribe_qos(&self) -> QoS {
       match self.subscribe.as_ref().unwrap().qos{
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
       }
    }

    pub fn get_publish_real_topic<'a>(&'a self, key_value: Option<&str>) -> Cow<'a, str> {
        wrap_real_topic(&self.publish, key_value)
    }

    pub fn get_subscribe_real_topic<'a>(&'a self, key_value: Option<&str>) -> Cow<'a, str> {
        if let Some(topic) = &self.subscribe {
            wrap_real_topic(topic, key_value)
        } else {
            Cow::Borrowed("")
        }
    }
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TopicConfig {
    pub register: Option<TopicWrap>,
    pub data: Option<TopicWrap>,
}

impl TopicConfig {
    pub fn get_register_topic(&self) -> Option<&str> {
        self.register
            .as_ref()
            .map(|topic| topic.get_publish_topic())
    }
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TcpConfig {}
