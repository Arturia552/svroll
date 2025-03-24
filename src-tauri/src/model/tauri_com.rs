use crate::{
    benchmark_param::{init_mqtt_context, read_from_csv_into_struct, Protocol},
    model::Rs2JsEntity,
    tcp::tcp_client::{TcpClient, TcpClientContext, TcpSendData},
    AsyncProcInputTx, MQTT_CLIENT_CONTEXT, TCP_CLIENT_CONTEXT,
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use once_cell::sync::OnceCell;
use tauri::{command, State};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex, task::JoinHandle, time::sleep};
use tracing::info;

use crate::{
    benchmark_param::BenchmarkConfig,
    mqtt::{basic::TopicConfig, Client},
    MqttClientData, MqttSendData,
};

use super::{connect_param::ConnectParam, Rs2JsMsgType};

static TASK: OnceCell<Arc<Mutex<Task>>> = OnceCell::new();

#[derive(Debug)]
pub struct Task {
    pub task_handle: Option<Vec<JoinHandle<()>>>,
    pub count_handle: Option<JoinHandle<()>>,
    pub status: Arc<AtomicBool>,
    pub counter: Arc<AtomicU32>,
}

#[command]
pub async fn receive_file(file_path: String) -> Result<String, String> {
    info!(file_path);
    Ok("选择成功".to_string())
}

#[command]
pub async fn write_file(file_path: String, content: String) -> Result<(), String> {
    fs::write(&file_path, content)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn load_config(file_path: String) -> Result<ConnectParam, String> {
    let config_str = fs::read_to_string(file_path)
        .await
        .map_err(|e| e.to_string())?;
    let config = serde_json::from_str(&config_str).map_err(|e| e.to_string())?;
    info!(?config);
    Ok(config)
}

#[command]
pub async fn start_task(
    param: ConnectParam,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    match param.protocol {
        Protocol::Mqtt => {
            let config = param.into_config().await.unwrap();
            start_mqtt(config, param.topic_config, async_proc_output_tx).await?;
        }
        Protocol::Tcp => {
            let config = param.into_tcp_config().await.unwrap();
            start_tcp(config, async_proc_output_tx).await?;
        }
    }
    Ok("开始发送消息...".to_string())
}

#[command]
pub async fn process_client_file(file_path: String) -> Result<Vec<MqttClientData>, String> {
    info!(file_path);
    let client_data: Vec<MqttClientData> = read_from_csv_into_struct(file_path.as_str())
        .await
        .map_err(|e| e.to_string())?;
    Ok(client_data)
}

async fn start_mqtt(
    param: BenchmarkConfig<MqttSendData, MqttClientData>,
    topic_config: Option<TopicConfig>,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    let tx = async_proc_output_tx.inner.lock().await.clone();
    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "开始初始化MQTT客户端...".to_string(),
    ))
    .await
    .unwrap();
    info!("开始初始化MQTT客户端...");
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
    // 等待所有客户端连接成功
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
        task.lock().await.task_handle = Some(task_handle);
    });
    let tx_clone = tx.clone();
    let count_handle = tokio::spawn(async move {
        let counter = task.lock().await.counter.clone();
        loop {
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

    Ok("开始发送消息...".to_string())
}

async fn start_tcp(
    benchmark_config: BenchmarkConfig<TcpSendData, TcpClient>,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    let tx = async_proc_output_tx.inner.lock().await.clone();

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
            count_handle: None,
            status: Arc::new(AtomicBool::new(true)),
            counter: Arc::new(AtomicU32::new(0)),
        }))
    });

    reset_task(task.clone()).await;

    let tx = async_proc_output_tx.inner.lock().await.clone();

    tokio::spawn(async move {
        let task = task.clone();
        let handles = tcp_client
            .spawn_message(clients, &*task.lock().await, &benchmark_config)
            .await
            .unwrap();
        task.lock().await.task_handle = Some(handles);
    });

    let tx_clone = tx.clone();

    let count_handle = tokio::spawn(async move {
        let counter = task.lock().await.counter.clone();
        loop {
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

    Ok("开始发送消息...".to_string())
}

#[command]
pub async fn stop_task(
    protocol: Option<Protocol>,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    if let Some(task) = TASK.get() {
        let tx = async_proc_output_tx.inner.lock().await.clone();
        // 第一步：停止任务状态，避免新的消息发送
        let mut task_lock = task.lock().await;
        task_lock.status.store(false, Ordering::SeqCst);
        info!("已将任务状态设置为停止");

        tx.send(Rs2JsEntity::new(
            Rs2JsMsgType::Terminal,
            "任务已停止，正在断开连接...".to_string(),
        ))
        .await
        .unwrap();

        // 根据协议类型处理不同的连接断开逻辑
        match protocol {
            Some(Protocol::Mqtt) | None => {
                // MQTT 连接断开逻辑
                info!("正在终止所有 MQTT 事件循环...");
                tx.send(Rs2JsEntity::new(
                    Rs2JsMsgType::Terminal,
                    "正在终止所有 MQTT 事件循环...".to_string(),
                ))
                .await
                .unwrap();

                for entry in MQTT_CLIENT_CONTEXT.iter() {
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

                for entry in MQTT_CLIENT_CONTEXT.iter() {
                    let client_entry = entry.value().clone();
                    disconnect_futures.push(tokio::spawn(async move {
                        let _ = client_entry.safe_disconnect().await;
                    }));
                }

                // 等待所有断开连接的任务完成（最多等待 5 秒）
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
                MQTT_CLIENT_CONTEXT.clear();
            },
            Some(Protocol::Tcp) => {
                // TCP 连接断开逻辑
                for mut entry in TCP_CLIENT_CONTEXT.iter_mut() {
                    let client = entry.value_mut();
                    let _ = client.shutdown().await;
                }
            }
        }

        // 停止任务句柄，适用于所有类型的任务
        if let Some(handles) = task_lock.task_handle.take() {
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
    // 清理可能残留的上下文
    match protocol {
        Some(Protocol::Mqtt) | None => MQTT_CLIENT_CONTEXT.clear(),
        Some(Protocol::Tcp) => TCP_CLIENT_CONTEXT.clear(),
    }
    
    Err("没有正在运行的任务".to_string())
}

async fn reset_task(task: Arc<Mutex<Task>>) {
    let task = task.lock().await;
    task.status.store(true, Ordering::SeqCst);
    task.counter.store(0, Ordering::SeqCst);
}

#[command]
pub async fn get_mqtt_clients() -> Result<Vec<MqttClientData>, String> {
    Ok(MQTT_CLIENT_CONTEXT
        .iter()
        .map(|entry| entry.value().clone())
        .collect())
}
