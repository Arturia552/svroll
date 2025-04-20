use crate::{
    context::{self, get_app_state},
    model::{database::HistoryConfig, Rs2JsEntity},
    param::{init_mqtt_context, read_from_csv_into_struct, Protocol},
    tcp::tcp_client::{TcpClient, TcpClientContext, TcpSendData},
    AsyncProcInputTx,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use serde_json::Value;
use tauri::{command, State};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex, task::JoinHandle, time::sleep};
use tracing::{error, info};

use crate::{
    mqtt::{basic::TopicConfig, Client},
    param::BasicConfig,
    MqttClientData, MqttSendData,
};

use super::{connect_param::ConnectParam, Rs2JsMsgType};

/// 全局任务单例，用于管理当前运行的任务实例
static TASK: OnceCell<Arc<Mutex<Task>>> = OnceCell::new();

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

/// 获取或初始化任务实例
async fn get_or_init_task() -> &'static Arc<Mutex<Task>> {
    TASK.get_or_init(|| {
        Arc::new(Mutex::new(Task {
            task_handle: None,
            message_handle: None,
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }))
    })
}

/// 记录日志并发送通知
async fn log_and_notify(
    tx: &tauri::async_runtime::Sender<Rs2JsEntity>,
    msg_type: Rs2JsMsgType,
    message: &str,
) -> Result<()> {
    info!("{}", message);
    tx.send(Rs2JsEntity::new(msg_type, message.to_string()))
        .await
        .context(format!("发送消息失败: {}", message))
}

/// 重置任务状态
///
/// 重置任务的状态和计数器
///
/// # 参数
/// * `task` - 要重置的任务
async fn reset_task(task: Arc<Mutex<Task>>) {
    let task = task.lock().await;
    task.status.store(true, Ordering::SeqCst);
    task.counter.store(0, Ordering::SeqCst);
}

/// 启动计数器任务
async fn spawn_counter(
    task: Arc<Mutex<Task>>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let counter = task.lock().await.counter.clone();
        let status = task.lock().await.status.clone();
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
    })
}

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
    fs::write(&file_path, content)
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
    let config_str = fs::read_to_string(&file_path)
        .await
        .with_context(|| format!("无法读取配置文件: {}", file_path))
        .map_err(|e| e.to_string())?;

    let config = serde_json::from_str(&config_str)
        .with_context(|| "配置文件格式错误")
        .map_err(|e| e.to_string())?;

    info!(?config);
    Ok(config)
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
pub async fn process_client_file(file_path: String) -> Result<Vec<MqttClientData>, String> {
    info!(file_path);
    let client_data: Vec<MqttClientData> = read_from_csv_into_struct(file_path.as_str())
        .await
        .with_context(|| format!("解析CSV文件失败: {}", file_path))
        .map_err(|e| e.to_string())?;
    Ok(client_data)
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
    let task = get_or_init_task().await;
    let tx: tauri::async_runtime::Sender<Rs2JsEntity> =
        async_proc_output_tx.inner.lock().await.clone();
    let param_clone = param.clone();

    // 启动主任务
    let handle = tokio::spawn(async move {
        match param.protocol {
            Protocol::Mqtt => {
                let topic_config = param.topic_config.clone();
                match param.into_config().await {
                    Ok(config) => {
                        if let Err(e) = start_mqtt(config, topic_config, tx.clone()).await {
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
                    if let Err(e) = start_tcp(config, tx.clone()).await {
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
    task.lock().await.task_handle = Some(handle);

    // 保存到数据库
    let save_result: Result<()> = async {
        let db = context::get_database().await;
        let db_lock = db.lock().await;
        let config = serde_json::to_value(&param_clone).context("配置序列化失败")?;
        let history_config =
            HistoryConfig::new(param_clone.protocol, &config).context("创建历史配置记录失败")?;
        db_lock
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

    Ok("开始发送消息...".to_string())
}

/// 启动MQTT客户端
///
/// 初始化MQTT配置、创建客户端连接并启动消息发送
///
/// # 参数
/// * `param` - MQTT配置参数
/// * `topic_config` - 主题配置
/// * `tx` - 状态消息发送通道
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
async fn start_mqtt(
    param: BasicConfig<MqttSendData, MqttClientData>,
    topic_config: Option<TopicConfig>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
) -> Result<String> {
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始初始化MQTT配置...").await?;

    let mqtt_config = topic_config.unwrap_or_else(TopicConfig::default);
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "初始化MQTT客户端成功").await?;

    let mqtt_client = init_mqtt_context(&param, mqtt_config).context("初始化MQTT上下文失败")?;

    log_and_notify(&tx, Rs2JsMsgType::Terminal, "初始化客户端...").await?;
    let mut clients = mqtt_client
        .setup_clients(&param)
        .await
        .context("设置MQTT客户端失败")?;

    log_and_notify(&tx, Rs2JsMsgType::Terminal, "等待连接...").await?;
    mqtt_client.wait_for_connections(&mut clients).await;
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "客户端已全部连接!").await?;

    // 获取任务实例并重置
    let task = get_or_init_task().await;
    reset_task(task.clone()).await;
    let message_task = task.clone();
    // 启动消息发送
    tokio::spawn(async move {
        info!("开始发送消息...");
        let task_guard = message_task.lock().await;
        match mqtt_client
            .spawn_message(clients, &task_guard, &param)
            .await
        {
            Ok(task_handle) => {
                drop(task_guard); 
                message_task.lock().await.message_handle = Some(task_handle);
            }
            Err(e) => {
                error!("启动消息发送任务失败: {}", e);
            }
        }
    });

    // 启动计数器
    let count_handle = spawn_counter(task.clone(), tx.clone()).await;
    task.lock().await.count_handle = Some(count_handle);

    // 通知开始发送消息
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始发送消息...").await?;

    Ok("".to_string())
}

/// 启动TCP客户端
///
/// 初始化TCP配置、创建客户端连接并启动消息发送
///
/// # 参数
/// * `benchmark_config` - TCP配置参数
/// * `tx` - 状态消息发送通道
///
/// # 返回
/// 成功返回Ok，失败返回错误信息
async fn start_tcp(
    benchmark_config: BasicConfig<TcpSendData, TcpClient>,
    tx: tauri::async_runtime::Sender<Rs2JsEntity>,
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

    // 获取任务实例并重置
    let task = get_or_init_task().await;
    reset_task(task.clone()).await;

    // 启动消息发送
    let message_task = task.clone();
    tokio::spawn(async move {
        let task_guard = message_task.lock().await;
        match tcp_client
            .spawn_message(clients, &task_guard, &benchmark_config)
            .await
        {
            Ok(handles) => {
                drop(task_guard); 
                message_task.lock().await.message_handle = Some(handles);
            }
            Err(e) => {
                error!("启动TCP消息发送任务失败: {}", e);
            }
        }
    });

    // 启动计数器
    let count_handle = spawn_counter(task.clone(), tx.clone()).await;
    task.lock().await.count_handle = Some(count_handle);

    // 通知开始发送消息
    log_and_notify(&tx, Rs2JsMsgType::Terminal, "开始发送消息...").await?;

    Ok("".to_string())
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
    let task = match TASK.get() {
        Some(task) => task,
        None => {
            info!("没有找到运行中的任务");
            if let Some(proto) = protocol {
                app_state.clear_clients(proto);
            }
            return Err("没有正在运行的任务".to_string());
        }
    };
    // 获取任务锁
    let mut task_lock = task.lock().await;

    // 检查任务状态
    if !task_lock.status.load(Ordering::SeqCst) {
        return Ok("无正在运行的任务".to_string());
    }

    // 中止主任务
    if let Some(handle) = task_lock.task_handle.take() {
        handle.abort();
    }

    task_lock.status.store(false, Ordering::SeqCst);
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
            let _ = log_and_notify(&tx, Rs2JsMsgType::Terminal, "正在终止所有 MQTT 事件循环...")
                .await
                .map_err(|e| {
                    error!("发送通知失败: {:#}", e);
                    e.to_string()
                });

            // 中止所有事件循环
            for entry in app_state.mqtt_clients().iter() {
                if let Some(event_handle) = &entry.value().event_loop_handle {
                    if let Some(handle) = event_handle.lock().await.take() {
                        handle.abort();
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;

            let _ = log_and_notify(
                &tx,
                Rs2JsMsgType::Terminal,
                "正在断开所有 MQTT 客户端连接...",
            )
            .await
            .map_err(|e| {
                error!("发送通知失败: {:#}", e);
                e.to_string()
            });

            let mut disconnect_futures = Vec::new();

            for entry in app_state.mqtt_clients().iter() {
                let client_entry = entry.value().clone();
                disconnect_futures.push(tokio::spawn(async move {
                    let _ = client_entry.safe_disconnect().await;
                }));
            }

            if !disconnect_futures.is_empty() {
                info!("等待所有断开连接操作完成...");
                let _ = tokio::time::timeout(Duration::from_secs(5), async {
                    for future in disconnect_futures {
                        let _ = future.await;
                    }
                })
                .await;
            }

            // 清理客户端上下文
            let _ = log_and_notify(&tx, Rs2JsMsgType::Terminal, "清理 MQTT 客户端上下文...")
                .await
                .map_err(|e| {
                    error!("发送通知失败: {:#}", e);
                    e.to_string()
                });
            app_state.mqtt_clients().clear();
        }
        Some(Protocol::Tcp) => {
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
        }
    }

    if let Some(handles) = task_lock.message_handle.take() {
        info!("正在停止消息发送任务句柄...");
        for h in handles {
            h.abort();
        }
    }

    sleep(Duration::from_secs(1)).await;

    if let Some(handle) = task_lock.count_handle.take() {
        info!("正在停止计数器句柄...");
        handle.abort();
    }

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
pub async fn get_clients(protocol: Protocol) -> Result<Vec<Value>, String> {
    match protocol {
        Protocol::Mqtt => {
            let client = get_app_state().get_mqtt_client_list();
            let client_json: Vec<Value> = client
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
            let client_json: Vec<Value> = client
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
