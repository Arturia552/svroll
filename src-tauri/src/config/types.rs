use serde::{Deserialize, Serialize};

/// 通信协议类型
///
/// 定义系统支持的通信协议类型
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Protocol {
    /// MQTT 协议
    Mqtt,
    /// TCP 协议
    Tcp,
}

/// 布尔标志枚举
///
/// 用于配置中表示布尔值的可序列化枚举
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Flag {
    /// 表示真/启用
    True,
    /// 表示假/禁用
    False,
}
