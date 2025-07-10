use once_cell::sync::OnceCell;
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::{sync::RwLock, time::sleep};
use tracing::{error, info};

use crate::{model::Rs2JsEntity, Rs2JsMsgType};

use super::types::Task;

/// 全局任务单例，用于管理当前运行的任务实例
pub static TASK: OnceCell<Arc<RwLock<Task>>> = OnceCell::new();

/// 获取或初始化任务实例
pub async fn get_or_init_task() -> &'static Arc<RwLock<Task>> {
    TASK.get_or_init(|| Arc::new(RwLock::new(Task::new())))
}

/// 重置任务状态
///
/// 重置任务的状态和计数器
///
/// # 参数
/// * `task` - 要重置的任务
pub async fn reset_task(task: Arc<RwLock<Task>>) {
    let task = task.read().await;
    task.status.store(true, Ordering::SeqCst);
    task.counter.store(0, Ordering::SeqCst);
    info!("任务状态已重置");
}

/// 启动计数器任务
///
/// 创建一个定时器，定期发送计数器状态到前端
///
/// # 参数
/// * `task` - 任务实例
/// * `tx` - 消息发送通道
///
/// # 返回
/// 计数器任务句柄
pub async fn spawn_counter(
    task: Arc<RwLock<Task>>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
) -> tokio::task::JoinHandle<()> {
    // 直接使用原子状态，避免锁竞争
    let task_read = task.read().await;
    let counter = task_read.counter.clone();
    let status = task_read.status.clone();
    drop(task_read); // 提早释放读锁

    tokio::spawn(async move {
        loop {
            if !status.load(Ordering::SeqCst) {
                break;
            }

            if let Err(e) = tx
                .send(Rs2JsEntity::new(
                    Rs2JsMsgType::Counter,
                    counter.load(Ordering::SeqCst).to_string(),
                ))
                .await
            {
                error!("发送计数器消息失败: {}", e);
            }
            sleep(Duration::from_secs(1)).await;
        }
        info!("计数器任务结束");
    })
}
