use std::fmt::Debug;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::{param::BasicConfig, model::task_com::Task};

/// 通用客户端接口，定义了客户端需要实现的基本功能
/// 
/// 泛型参数:
/// - T: 可调试的数据类型，用于客户端数据传输
/// - C: 客户端数据类型，用于存储连接信息
pub trait Client<T, C>: Send + Sync
where
    T: Debug,
{
    /// 客户端实例类型
    type Item;

    /// 设置并初始化客户端
    /// 
    /// 根据配置创建一组客户端连接
    /// 
    /// # 参数
    /// * `config` - 包含连接参数的配置对象
    /// 
    /// # 返回
    /// 成功时返回客户端实例列表，失败时返回错误
    fn setup_clients(
        &self,
        config: &BasicConfig<T, C>,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Item>, Error>> + Send;

    /// 等待所有客户端连接完成
    /// 
    /// # 参数
    /// * `clients` - 需要等待连接的客户端列表
    fn wait_for_connections(
        &self,
        clients: &mut [Self::Item],
    ) -> impl std::future::Future<Output = ()> + Send;

    /// 处理客户端连接成功后的操作
    /// 
    /// # 参数
    /// * `client` - 已连接的客户端实例
    /// 
    /// # 返回
    /// 操作成功返回Ok，失败返回错误
    fn on_connect_success(
        &self,
        client: &mut Self::Item,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    
    /// 启动消息发送任务
    /// 
    /// # 参数
    /// * `clients` - 用于发送消息的客户端列表
    /// * `task` - 任务信息
    /// * `config` - 配置参数
    /// 
    /// # 返回
    /// 成功时返回任务句柄列表，失败时返回错误
    fn spawn_message(
        &self,
        clients: Vec<Self::Item>,
        task: &Task,
        config: &BasicConfig<T, C>,
    ) -> impl std::future::Future<Output = Result<Vec<JoinHandle<()>>, Error>> + Send;
}

/// 连接状态枚举
/// 
/// 表示客户端的连接状态:
/// - Connected: 已连接，表示连接成功且可以通信
/// - Connecting: 连接中，表示正在尝试建立连接
/// - Failed: 连接失败，表示连接尝试失败或连接中断
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Failed,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Connecting
    }
}