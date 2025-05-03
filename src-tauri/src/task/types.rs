use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};
use tokio::task::JoinHandle;
use tracing::debug;

/// 任务结构体
///
/// 管理通信任务的执行状态和相关句柄，包括主任务、消息发送任务和计数任务
#[derive(Debug)]
pub struct Task {
    /// 主任务句柄，负责整体任务协调
    pub task_handle: Option<JoinHandle<()>>,
    /// 消息发送任务句柄列表，负责具体的消息发送
    pub message_handle: Option<Vec<JoinHandle<()>>>,
    /// 计数任务句柄，负责统计和报告已发送消息数
    pub count_handle: Option<JoinHandle<()>>,
    /// 任务执行状态标志，true表示正在运行，false表示已停止
    pub status: Arc<AtomicBool>,
    /// 消息计数器，记录已发送的消息数量
    pub counter: Arc<AtomicU32>,
}

impl Task {
    /// 创建新的任务实例
    pub fn new() -> Self {
        debug!("创建新的任务实例");
        Task {
            task_handle: None,
            message_handle: None,
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// 获取计数器引用
    pub fn counter(&self) -> Arc<AtomicU32> {
        self.counter.clone()
    }

    /// 获取状态引用
    pub fn status(&self) -> Arc<AtomicBool> {
        self.status.clone()
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}
