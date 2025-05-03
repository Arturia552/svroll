use rumqttc::Packet;
use std::{collections::HashMap, fmt::Debug};

/// MQTT消息处理上下文
///
/// 包含MQTT消息处理期间所需的所有信息
#[derive(Clone, Debug)]
pub struct MqttHookContext {
    /// MQTT包
    pub packet: Option<Packet>,
    /// 主题路径
    pub topic: Option<String>,
    /// 消息内容
    pub payload: Option<Vec<u8>>,
    /// 元数据，用于在钩子处理器之间传递额外信息
    metadata: HashMap<String, String>,
}

impl MqttHookContext {
    /// 创建新的钩子上下文
    pub fn new(packet: Packet, topic: String, payload: Vec<u8>) -> Self {
        Self {
            packet: Some(packet),
            topic: Some(topic),
            payload: Some(payload),
            metadata: HashMap::new(),
        }
    }

    /// 创建空的钩子上下文
    pub fn empty() -> Self {
        Self {
            packet: None,
            topic: None,
            payload: None,
            metadata: HashMap::new(),
        }
    }

    /// 获取MQTT包
    pub fn get_packet(&self) -> Option<&Packet> {
        self.packet.as_ref()
    }

    /// 获取主题
    pub fn get_topic(&self) -> Option<&String> {
        self.topic.as_ref()
    }

    /// 获取载荷
    pub fn get_payload(&self) -> Option<&Vec<u8>> {
        self.payload.as_ref()
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// MQTT钩子处理结果
#[derive(Debug, Clone)]
pub enum MqttHookResult {
    /// 继续处理链中的下一个处理器
    Continue(MqttHookContext),
    /// 停止处理链，使用修改后的上下文
    Terminate(MqttHookContext),
    /// 忽略当前消息（等同于直接中止，但更明确表明处理器决定跳过此消息）
    Ignore,
}

impl MqttHookResult {
    /// 检查是否应继续处理链
    pub fn should_continue(&self) -> bool {
        matches!(self, MqttHookResult::Continue(_))
    }

    /// 提取上下文（如果有）
    pub fn context(self) -> Option<MqttHookContext> {
        match self {
            MqttHookResult::Continue(ctx) => Some(ctx),
            MqttHookResult::Terminate(ctx) => Some(ctx),
            MqttHookResult::Ignore => None,
        }
    }
}
