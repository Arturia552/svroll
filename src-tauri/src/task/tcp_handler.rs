use anyhow::{Context, Result};
use tokio::{io::AsyncWriteExt, sync::RwLock};
use tracing::{error, info};
use std::sync::Arc;

use crate::{
    model::Rs2JsEntity, mqtt::Client, param::BasicConfig, tcp::tcp_client::{TcpClient, TcpClientContext, TcpSendData}, Rs2JsMsgType
};

use super::{
    types::Task,
    utils::log_and_notify,
};

/// 启动TCP客户端
///
/// 初始化TCP配置、创建客户端连接并启动消息发送
///
/// # 参数
/// * `benchmark_config` - TCP配置参数
/// * `tx` - 状态消息发送通道
/// * `task` - 任务实例
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
pub async fn start_tcp(
    benchmark_config: BasicConfig<TcpSendData, TcpClient>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
    task: Arc<RwLock<Task>>,
) -> Result<String> {
    let tcp_client = TcpClientContext::new(
        Arc::new(benchmark_config.send_data.clone()),
        benchmark_config.enable_register,
    );

    let mut clients = tcp_client
        .setup_clients(&benchmark_config)
        .await
        .context("设置TCP客户端失败")?;

    log_and_notify(&tx, Rs2JsMsgType::Terminal, "等待连接...").await?;
    tcp_client.wait_for_connections(&mut clients).await;
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "客户端已全部连接!").await?;

    // 启动消息发送
    tokio::spawn(async move {
        // 使用写锁来更新任务句柄
        let task_read = task.read().await;
        match tcp_client
            .spawn_message(clients, &task_read, &benchmark_config)
            .await
        {
            Ok(handles) => {
                // 释放读锁后再获取写锁
                drop(task_read);
                let mut task_write = task.write().await;
                task_write.message_handle = Some(handles);
                info!("TCP消息发送任务启动成功");
            }
            Err(e) => {
                error!("启动TCP消息发送任务失败: {}", e);
            }
        }
    });

    // 通知开始发送消息
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始发送消息...").await?;

    Ok("TCP消息发送任务启动成功".to_string())
}

/// 停止TCP客户端
///
/// 停止所有TCP连接并清理资源
///
/// # 参数
/// * `app_state` - 应用状态
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
pub async fn stop_tcp_clients(app_state: &crate::state::AppState) -> Result<()> {
    // 关闭所有TCP连接
    for mut entry in app_state.tcp_clients().iter_mut() {
        let client_ref = entry.value_mut();
        if let Some(writer) = client_ref.1.as_mut() {
            if let Err(e) = writer.shutdown().await {
                error!("TCP连接关闭失败: {:#}", e);
            }
        }
    }

    // 清理TCP客户端
    app_state.tcp_clients().clear();
    info!("已停止并清理所有TCP客户端");

    Ok(())
}
