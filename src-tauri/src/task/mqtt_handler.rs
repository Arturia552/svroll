use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::{
    model::Rs2JsEntity,
    mqtt::{basic::TopicConfig, Client},
    param::{init_mqtt_context, BasicConfig},
    MqttClientData, MqttSendData, Rs2JsMsgType,
};

use super::{types::Task, utils::log_and_notify};

/// 启动MQTT客户端
///
/// 初始化MQTT配置、创建客户端连接并启动消息发送
///
/// # 参数
/// * `param` - MQTT配置参数
/// * `topic_config` - 主题配置
/// * `tx` - 状态消息发送通道
/// * `task` - 任务实例
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
pub async fn start_mqtt(
    param: BasicConfig<MqttSendData, MqttClientData>,
    topic_config: Option<TopicConfig>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
    task: Arc<RwLock<Task>>,
) -> Result<String> {
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始初始化MQTT配置...").await?;

    let mqtt_config = topic_config.unwrap_or_else(TopicConfig::default);
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "初始化MQTT客户端成功").await?;

    let mqtt_client = init_mqtt_context(&param, mqtt_config).await.context("初始化MQTT上下文失败")?;

    log_and_notify(&tx, Rs2JsMsgType::Terminal, "初始化客户端...").await?;
    let mut clients = mqtt_client
        .setup_clients(&param)
        .await
        .context("设置MQTT客户端失败")?;

    log_and_notify(&tx, Rs2JsMsgType::Terminal, "等待连接...").await?;
    mqtt_client.wait_for_connections(&mut clients).await;
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "客户端已全部连接!").await?;

    // 启动消息发送
    tokio::spawn(async move {
        info!("开始发送消息...");
        // 使用读锁获取任务状态
        let task_read = task.read().await;
        match mqtt_client.spawn_message(clients, &task_read, &param).await {
            Ok(task_handle) => {
                // 释放读锁并获取写锁来更新任务句柄
                drop(task_read);
                let mut task_write = task.write().await;
                task_write.message_handle = Some(task_handle);
                info!("MQTT消息发送任务启动成功");
            }
            Err(e) => {
                error!("启动消息发送任务失败: {}", e);
            }
        }
    });

    // 通知开始发送消息
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始发送消息...").await?;

    Ok("MQTT消息发送任务启动成功".to_string())
}

/// 停止MQTT客户端
///
/// 停止所有MQTT连接并清理资源
///
/// # 参数
/// * `app_state` - 应用状态
/// * `tx` - 消息发送通道
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
pub async fn stop_mqtt_clients(
    app_state: &crate::state::AppState,
    tx: &tauri::async_runtime::Sender<Rs2JsEntity>,
) -> Result<()> {
    log_and_notify(tx, Rs2JsMsgType::Terminal, "正在终止所有 MQTT 事件循环...").await?;

    // 中止所有事件循环
    for entry in app_state.mqtt_clients().iter() {
        if let Some(event_handle) = &entry.value().event_loop_handle {
            // 使用读锁来获取句柄并执行abort操作
            {
                let event_read = event_handle.read().await;
                if let Some(handle) = event_read.as_ref() {
                    handle.abort();
                }
            } // 读锁在这里被释放

            // 获取写锁来清理句柄
            let mut event_write = event_handle.write().await;
            *event_write = None;
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    log_and_notify(
        tx,
        Rs2JsMsgType::Terminal,
        "正在断开所有 MQTT 客户端连接...",
    )
    .await?;

    let mut disconnect_futures = Vec::new();

    for entry in app_state.mqtt_clients().iter() {
        let client_entry = entry.value().clone();
        disconnect_futures.push(tokio::spawn(async move {
            let _ = client_entry.safe_disconnect().await;
        }));
    }

    if !disconnect_futures.is_empty() {
        info!("等待所有断开连接操作完成...");
        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
            for future in disconnect_futures {
                let _ = future.await;
            }
        })
        .await;
    }

    // 清理客户端上下文
    log_and_notify(tx, Rs2JsMsgType::Terminal, "清理 MQTT 客户端上下文...").await?;
    app_state.mqtt_clients().clear();

    Ok(())
}
