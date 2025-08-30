use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use super::types::Protocol;

/// 客户端手动输入的配置信息
///
/// 包含连接、发送和性能参数的综合配置
#[derive(Debug, Serialize, serde::Deserialize, Clone)]
pub struct BasicConfig<T, C> {
    /// 设置需发送的数据内容
    pub send_data: Arc<T>,

    /// 使用的数据传输协议，默认为mqtt
    pub protocol_type: Protocol,

    /// 客户端配置列表
    pub clients: Arc<Vec<C>>,

    /// 设置启动协程数量,默认为200
    pub thread_size: usize,

    /// 是否启用随机值
    pub enable_random: bool,

    /// 设置broker地址,默认为mqtt://localhost:1883
    pub broker: Arc<String>,

    /// 每秒最多启动连接数
    pub max_connect_per_second: usize,

    /// 设置发送间隔,默认为1秒
    pub send_interval: u64,
}

impl<T, C> BasicConfig<T, C>
where
    T: DeserializeOwned + Debug,
    C: DeserializeOwned + Debug,
{
    /// 创建新的基准测试配置
    ///
    /// # 参数
    /// * `send_data` - 要发送的数据内容
    /// * `clients` - 客户端配置列表
    /// * `protocol_type` - 使用的协议类型
    /// * `thread_size` - 线程数量
    /// * `enable_random` - 是否启用随机值
    /// * `broker` - 服务器地址
    /// * `max_connect_per_second` - 每秒最大连接数
    /// * `send_interval` - 发送间隔(秒)
    pub fn new(
        send_data: T,
        clients: Vec<C>,
        protocol_type: Protocol,
        thread_size: usize,
        enable_random: bool,
        broker: String,
        max_connect_per_second: usize,
        send_interval: u64,
    ) -> Self {
        Self {
            send_data: Arc::new(send_data),
            protocol_type,
            clients: Arc::new(clients),
            thread_size,
            enable_random,
            broker: Arc::new(broker),
            max_connect_per_second,
            send_interval,
        }
    }

    /// 验证配置参数的有效性
    ///
    /// 检查配置中的必要参数是否有效，防止使用无效配置
    ///
    /// # 返回
    /// 成功返回Ok，配置无效返回错误信息
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

    /// 设置发送数据内容
    ///
    /// # 参数
    /// * `data` - 要设置的数据内容
    pub fn set_send_data(&mut self, data: T) {
        self.send_data = Arc::new(data);
    }

    /// 获取发送数据内容的Arc引用
    pub fn get_send_data_arc(&self) -> &Arc<T> {
        &self.send_data
    }

    /// 获取发送数据内容
    pub fn get_send_data(&self) -> &T {
        &self.send_data
    }

    /// 获取服务器地址
    pub fn get_broker(&self) -> &str {
        &self.broker
    }

    /// 获取客户端配置列表
    pub fn get_clients(&self) -> &Vec<C> {
        &self.clients
    }

    /// 获取客户端配置列表的Arc引用
    pub fn get_clients_arc(&self) -> &Arc<Vec<C>> {
        &self.clients
    }

    /// 获取每秒最大连接数
    pub fn get_max_connect_per_second(&self) -> usize {
        self.max_connect_per_second
    }

    /// 设置发送间隔
    ///
    /// # 参数
    /// * `send_interval` - 发送间隔(秒)
    pub fn set_send_interval(&mut self, send_interval: u64) {
        self.send_interval = send_interval;
    }
}
