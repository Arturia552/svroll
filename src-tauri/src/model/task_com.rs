use crate::{
    param::{init_mqtt_context, read_from_csv_into_struct, Protocol},
    context::{self, get_app_state},
    model::{database::HistoryConfig, Rs2JsEntity},
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

use once_cell::sync::OnceCell;
use serde_json::Value;
use tauri::{command, State};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex, task::JoinHandle, time::sleep};
use tracing::info;

use crate::{
    param::BasicConfig,
    mqtt::{basic::TopicConfig, Client},
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
    let config_str = fs::read_to_string(file_path)
        .await
        .map_err(|e| e.to_string())?;
    let config = serde_json::from_str(&config_str).map_err(|e| e.to_string())?;
    info!(?config);
    Ok(config)
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
    let task = TASK.get_or_init(|| {
        Arc::new(Mutex::new(Task {
            task_handle: None,
            message_handle: None,
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }))
    });
    let tx: tauri::async_runtime::Sender<Rs2JsEntity> =
        async_proc_output_tx.inner.lock().await.clone();
    let param_clone = param.clone();
    let handle = tokio::spawn(async move {
        match param.protocol {
            Protocol::Mqtt => {
                let topic_config = param.topic_config.clone();
                let config = param.into_config().await.unwrap();

                start_mqtt(config, topic_config, tx).await.unwrap();
            }
            Protocol::Tcp => {
                let config = param.into_tcp_config().await.unwrap();
                start_tcp(config, tx).await.unwrap();
            }
        }
    });
    task.lock().await.task_handle = Some(handle);

    // 保存到数据库
    let db = context::get_database().await;
    let db_lock = db.lock().await;
    let config = serde_json::to_value(&param_clone).map_err(|e| e.to_string())?;
    let history_config = HistoryConfig::new(param_clone.protocol, &config).map_err(|e| e.to_string())?;
    db_lock
        .save_config(&history_config)
        .await
        .map_err(|e| e.to_string())?;

    Ok("开始发送消息...".to_string())
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
        .map_err(|e| e.to_string())?;
    Ok(client_data)
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
) -> Result<String, String> {
    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "开始初始化MQTT配置...".to_string(),
    ))
    .await
    .unwrap();
    info!("开始初始化MQTT配置...");
    let mqtt_config = topic_config.unwrap_or_else(|| TopicConfig::default());

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "初始化MQTT客户端成功".to_string(),
    ))
    .await
    .unwrap();
    info!("初始化MQTT客户端成功");
    let mqtt_client = init_mqtt_context(&param, mqtt_config).map_err(|e| e.to_string())?;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "初始化客户端...".to_string(),
    ))
    .await
    .unwrap();
    info!("初始化客户端...");
    let mut clients = mqtt_client
        .setup_clients(&param)
        .await
        .map_err(|e| e.to_string())?;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "等待连接...".to_string(),
    ))
    .await
    .unwrap();
    info!("等待连接...");
    mqtt_client.wait_for_connections(&mut clients).await;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "客户端已全部连接!".to_string(),
    ))
    .await
    .unwrap();
    info!("客户端已全部连接!");
    let task = TASK.get_or_init(|| {
        Arc::new(Mutex::new(Task {
            task_handle: None,
            message_handle: None,
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }))
    });

    reset_task(task.clone()).await;

    tokio::spawn(async move {
        info!("开始发送消息...");
        let task = task.clone();
        let task_handle = mqtt_client
            .spawn_message(clients, &*task.lock().await, &param)
            .await
            .unwrap();
        task.lock().await.message_handle = Some(task_handle);
    });
    let tx_clone = tx.clone();
    let count_handle = tokio::spawn(async move {
        let counter = task.lock().await.counter.clone();
        let status = task.lock().await.status.clone();
        loop {
            if !status.load(Ordering::SeqCst) {
                break;
            }

            tx_clone
                .send(Rs2JsEntity::new(
                    Rs2JsMsgType::Counter,
                    counter.load(Ordering::SeqCst).to_string(),
                ))
                .await
                .unwrap();
            sleep(Duration::from_secs(1)).await;
        }
    });

    let mut task = task.lock().await;
    task.count_handle = Some(count_handle);

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "开始发送消息...".to_string(),
    ))
    .await
    .unwrap();

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
) -> Result<String, String> {
    let tcp_client = TcpClientContext::new(
        Arc::new(benchmark_config.send_data.clone()),
        benchmark_config.enable_register,
    );
    let mut clients = tcp_client
        .setup_clients(&benchmark_config)
        .await
        .map_err(|e| e.to_string())?;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "等待连接...".to_string(),
    ))
    .await
    .unwrap();

    tcp_client.wait_for_connections(&mut clients).await;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "客户端已全部连接!".to_string(),
    ))
    .await
    .unwrap();

    let task = TASK.get_or_init(|| {
        Arc::new(Mutex::new(Task {
            task_handle: None,
            message_handle: None,
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }))
    });

    reset_task(task.clone()).await;

    tokio::spawn(async move {
        let task = task.clone();
        let handles = tcp_client
            .spawn_message(clients, &*task.lock().await, &benchmark_config)
            .await
            .unwrap();
        task.lock().await.message_handle = Some(handles);
    });

    let tx_clone = tx.clone();

    let count_handle = tokio::spawn(async move {
        let counter = task.lock().await.counter.clone();
        let status = task.lock().await.status.clone();
        loop {
            if !status.load(Ordering::SeqCst) {
                break;
            }

            tx.send(Rs2JsEntity::new(
                Rs2JsMsgType::Counter,
                counter.load(Ordering::SeqCst).to_string(),
            ))
            .await
            .unwrap();
            sleep(Duration::from_secs(1)).await;
        }
    });

    let mut task = task.lock().await;
    task.count_handle = Some(count_handle);

    tx_clone
        .send(Rs2JsEntity::new(
            Rs2JsMsgType::Terminal,
            "开始发送消息...".to_string(),
        ))
        .await
        .unwrap();

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

    if let Some(task) = TASK.get() {

        let tx = async_proc_output_tx.inner.lock().await.clone();
        let mut task_lock = task.lock().await;

        if task_lock.status.load(Ordering::SeqCst) == false {
            return Ok("无正在运行的任务".to_string());
        }

        task_lock.task_handle.take().map(|handle| {
            handle.abort();
        });

        task_lock.status.store(false, Ordering::SeqCst);
        info!("已将任务状态设置为停止");

        tx.send(Rs2JsEntity::new(
            Rs2JsMsgType::Terminal,
            "任务已停止，正在断开连接...".to_string(),
        ))
        .await
        .unwrap();

        match protocol {
            Some(Protocol::Mqtt) | None => {
                info!("正在终止所有 MQTT 事件循环...");
                tx.send(Rs2JsEntity::new(
                    Rs2JsMsgType::Terminal,
                    "正在终止所有 MQTT 事件循环...".to_string(),
                ))
                .await
                .unwrap();

                for entry in app_state.mqtt_clients().iter() {
                    if let Some(event_handle) = &entry.value().event_loop_handle {
                        if let Some(handle) = event_handle.lock().await.take() {
                            handle.abort();
                        }
                    }
                }

                sleep(Duration::from_millis(500)).await;

                info!("正在断开所有 MQTT 客户端连接...");
                tx.send(Rs2JsEntity::new(
                    Rs2JsMsgType::Terminal,
                    "正在断开所有 MQTT 客户端连接...".to_string(),
                ))
                .await
                .unwrap();

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

                info!("清理 MQTT 客户端上下文...");
                tx.send(Rs2JsEntity::new(
                    Rs2JsMsgType::Terminal,
                    "清理 MQTT 客户端上下文...".to_string(),
                ))
                .await
                .unwrap();
                app_state.mqtt_clients().clear();
            }
            Some(Protocol::Tcp) => {
                for mut entry in app_state.tcp_clients().iter_mut() {
                    let client_ref = entry.value_mut();
                    if let Some(writer) = client_ref.1.as_mut() {
                        let _ = writer.shutdown().await;
                    }
                }
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

        return Ok("任务已完全中断，所有连接已断开".to_string());
    }

    info!("没有找到运行中的任务");
    app_state.clear_clients(protocol.unwrap());

    Err("没有正在运行的任务".to_string())
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
                .map(|entry| serde_json::to_value(entry).unwrap())
                .collect();
            Ok(client_json)
        }
        Protocol::Tcp => {
            let client = get_app_state().get_tcp_client_list();
            let client_json: Vec<Value> = client
                .iter()
                .map(|entry| serde_json::to_value(entry).unwrap())
                .collect();
            Ok(client_json)
        }
    }
}
