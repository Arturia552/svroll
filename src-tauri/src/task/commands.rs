use anyhow::{Context as AnyhowContext, Result};
use tauri::{command, State};
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::{error, info};

use crate::model::connect_param::ConnectParam;
use crate::task::file_handler::CsvClientInfo;
use crate::MqttClientData;
use crate::Rs2JsMsgType;
use crate::{
    context::{self, get_app_state},
    model::{database::HistoryConfig, Rs2JsEntity},
    param::Protocol,
    AsyncProcInputTx,
};

use super::{
    file_handler::{load_config_file, process_csv_file, write_file_content},
    manager::{get_or_init_task, reset_task, spawn_counter},
    mqtt_handler::{start_mqtt, stop_mqtt_clients},
    tcp_handler::{start_tcp, stop_tcp_clients},
    utils::log_and_notify,
};

/// 接收文件命令
///
/// 处理前端选择的文件，并确认文件已被成功接收
///
/// # 参数
/// * `file_path` - 选择的文件路径
///
/// # 返回
/// 成功返回选择成功消息，失败返回错误信息
#[command]
pub async fn receive_file(file_path: String) -> Result<String, String> {
    info!(file_path);
    Ok("选择成功".to_string())
}

/// 写入文件命令
///
/// 将内容写入指定文件路径
///
/// # 参数
/// * `file_path` - 目标文件路径
/// * `content` - 要写入的文件内容
///
/// # 返回
/// 成功返回Ok(()), 失败返回错误信息
#[command]
pub async fn write_file(file_path: String, content: String) -> Result<(), String> {
    write_file_content(&file_path, &content)
        .await
        .map_err(|e| e.to_string())
}

/// 加载配置文件命令
///
/// 从指定路径加载JSON格式的连接配置文件
///
/// # 参数
/// * `file_path` - 配置文件路径
///
/// # 返回
/// 成功返回连接参数对象，失败返回错误信息
#[command]
pub async fn load_config(file_path: String) -> Result<ConnectParam, String> {
    load_config_file(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// 处理客户端CSV文件
///
/// 解析CSV文件中的客户端配置数据
///
/// # 参数
/// * `file_path` - CSV文件路径
///
/// # 返回
/// 成功返回客户端数据列表，失败返回错误信息
#[command]
pub async fn process_client_file(file_path: String) -> Result<Vec<CsvClientInfo>, String> {
    process_csv_file(&file_path)
        .await
        .map_err(|e| e.to_string())
}

/// 启动通信任务命令
///
/// 根据连接参数启动MQTT或TCP通信任务，并保存配置到数据库
///
/// # 参数
/// * `param` - 连接参数配置
/// * `async_proc_output_tx` - 异步消息发送通道
///
/// # 返回
/// 成功返回启动成功消息，失败返回错误信息
#[command]
pub async fn start_task(
    param: ConnectParam,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    info!("启动任务: {:?}", param);
    let task = get_or_init_task().await;
    let tx: tauri::async_runtime::Sender<Rs2JsEntity> =
        async_proc_output_tx.inner.lock().await.clone();
    let param_clone = param.clone();

    // 重置任务状态
    reset_task(task.clone()).await;

    // 启动计数器
    let count_handle = spawn_counter(task.clone(), tx.clone()).await;
    // 获取写锁更新任务句柄
    {
        let task_read = task.read().await;
        let mut handles = task_read.handles.write().await;
        handles.count_handle = Some(count_handle);
    }

    // 启动主任务
    let handle = tokio::spawn(async move {
        match param.protocol {
            Protocol::Mqtt => {
                let topic_config = param.topic_config.clone();
                match param.into_config().await {
                    Ok(config) => {
                        if let Err(e) =
                            start_mqtt(config, topic_config, tx.clone(), task.clone()).await
                        {
                            error!("MQTT 任务启动失败: {:#}", e);
                            let _ = log_and_notify(
                                &tx,
                                Rs2JsMsgType::Terminal,
                                &format!("MQTT 任务启动失败: {}", e),
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        error!("MQTT 配置创建失败: {:#}", e);
                        let _ = log_and_notify(
                            &tx,
                            Rs2JsMsgType::Terminal,
                            &format!("MQTT 配置创建失败: {}", e),
                        )
                        .await;
                    }
                }
            }
            Protocol::Tcp => match param.into_tcp_config().await {
                Ok(config) => {
                    if let Err(e) = start_tcp(config, tx.clone(), task.clone()).await {
                        error!("TCP 任务启动失败: {:#}", e);
                        let _ = log_and_notify(
                            &tx,
                            Rs2JsMsgType::Terminal,
                            &format!("TCP 任务启动失败: {}", e),
                        )
                        .await;
                    }
                }
                Err(e) => {
                    error!("TCP 配置创建失败: {:#}", e);
                    let _ = log_and_notify(
                        &tx,
                        Rs2JsMsgType::Terminal,
                        &format!("TCP 配置创建失败: {}", e),
                    )
                    .await;
                }
            },
        }
    });

    // 更新任务句柄
    {
        let task_read = task.read().await;
        let mut handles = task_read.handles.write().await;
        handles.task_handle = Some(handle);
    }

    // 保存到数据库
    let save_result: Result<()> = async {
        let db = context::get_database().await;
        // 使用读锁访问数据库
        let db_read = db.read().await;
        let config = serde_json::to_value(&param_clone).context("配置序列化失败")?;
        let history_config =
            HistoryConfig::new(param_clone.protocol, &config).context("创建历史配置记录失败")?;
        db_read
            .save_config(&history_config)
            .await
            .context("保存配置到数据库失败")?;
        Ok(())
    }
    .await;

    if let Err(e) = save_result {
        error!("保存配置到数据库失败: {:#}", e);
        // 继续执行，不影响主流程
    }

    Ok("开始执行任务...".to_string())
}

/// 停止通信任务命令
///
/// 停止当前运行的通信任务，断开所有连接，清理资源
///
/// # 参数
/// * `protocol` - 协议类型，指定要停止的任务类型
/// * `async_proc_output_tx` - 异步消息发送通道
///
/// # 返回
/// 成功返回停止成功消息，失败返回错误信息
#[command]
pub async fn stop_task(
    protocol: Option<Protocol>,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    let app_state = get_app_state();
    let tx = async_proc_output_tx.inner.lock().await.clone();
    info!("停止任务: {:?}", protocol);

    // 获取任务实例，如果不存在则返回错误
    let task = match super::manager::TASK.get() {
        Some(task) => task,
        None => {
            info!("没有找到运行中的任务");
            if let Some(proto) = protocol {
                app_state.clear_clients(proto);
            }
            return Err("没有正在运行的任务".to_string());
        }
    };

    // 读取任务状态
    {
        let task_read = task.read().await;
        // 检查任务状态
        if !task_read.status.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok("无正在运行的任务".to_string());
        }
    }

    // 更新任务状态和句柄
    let message_handles;
    let count_handle;
    {
        // 先更新原子状态
        let task_read = task.read().await;
        task_read
            .status
            .store(false, std::sync::atomic::Ordering::SeqCst);
        drop(task_read);

        // 再处理句柄
        let task_read = task.read().await;
        let mut handles = task_read.handles.write().await;
        // 中止主任务
        if let Some(handle) = handles.task_handle.take() {
            handle.abort();
        }

        // 保存任务句柄以便稍后中止
        message_handles = handles.message_handle.take();
        count_handle = handles.count_handle.take();
    }

    info!("已将任务状态设置为停止");

    let _ = log_and_notify(&tx, Rs2JsMsgType::Terminal, "任务已停止，正在断开连接...")
        .await
        .map_err(|e| {
            error!("发送停止通知失败: {:#}", e);
            e.to_string()
        });

    // 根据协议类型停止客户端
    match protocol {
        Some(Protocol::Mqtt) | None => {
            if let Err(e) = stop_mqtt_clients(&app_state, &tx).await {
                error!("停止MQTT客户端失败: {:#}", e);
            }
        }
        Some(Protocol::Tcp) => {
            if let Err(e) = stop_tcp_clients(&app_state).await {
                error!("停止TCP客户端失败: {:#}", e);
            }
        }
    }

    // 中止消息发送任务句柄
    if let Some(handles) = message_handles {
        info!("正在停止消息发送任务句柄...");
        for h in handles {
            h.abort();
        }
    }

    sleep(Duration::from_secs(1)).await;

    // 中止计数器任务句柄
    if let Some(handle) = count_handle {
        info!("正在停止计数器句柄...");
        handle.abort();
    }
    info!("所有任务已完全停止");
    Ok("任务已完全中断，所有连接已断开".to_string())
}

/// 获取客户端列表
///
/// 根据协议类型获取当前连接的客户端列表
///
/// # 参数
/// * `protocol` - 协议类型
///
/// # 返回
/// 成功返回客户端列表，失败返回错误信息
#[command]
pub async fn get_clients(protocol: Protocol) -> Result<Vec<serde_json::Value>, String> {
    match protocol {
        Protocol::Mqtt => {
            let client = get_app_state().get_mqtt_client_list();
            let client_json: Vec<serde_json::Value> = client
                .iter()
                .map(|entry| {
                    serde_json::to_value(entry)
                        .unwrap_or_else(|_| serde_json::json!({"error": "序列化失败"}))
                })
                .collect();
            Ok(client_json)
        }
        Protocol::Tcp => {
            let client = get_app_state().get_tcp_client_list();
            let client_json: Vec<serde_json::Value> = client
                .iter()
                .map(|entry| {
                    serde_json::to_value(entry)
                        .unwrap_or_else(|_| serde_json::json!({"error": "序列化失败"}))
                })
                .collect();
            Ok(client_json)
        }
    }
}
