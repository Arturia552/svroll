use crate::{
    benchmark_param::{init_mqtt_context, read_from_csv_into_struct, Protocol},
    model::Rs2JsEntity,
    tcp::tcp_client::{TcpClient, TcpClientData},
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
use tauri::{command, State};
use tokio::{fs, sync::Mutex, task::JoinHandle, time::sleep};
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
    info!(file_path);

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
    info!(?param);
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

    let mqtt_config = topic_config.unwrap_or_else(|| TopicConfig::default());

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "初始化MQTT客户端成功".to_string(),
    ))
    .await
    .unwrap();

    let mqtt_client = init_mqtt_context(&param, mqtt_config).map_err(|e| e.to_string())?;

    tx.send(Rs2JsEntity::new(
        Rs2JsMsgType::Terminal,
        "初始化客户端...".to_string(),
    ))
    .await
    .unwrap();

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

    // 等待所有客户端连接成功
    mqtt_client.wait_for_connections(&mut clients).await;

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

    tokio::spawn(async move {
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
    benchmark_config: BenchmarkConfig<Vec<u8>, TcpClientData>,
    async_proc_output_tx: State<'_, AsyncProcInputTx>,
) -> Result<String, String> {
    let tcp_client = TcpClient::new(
        Arc::new(benchmark_config.send_data.clone()),
        benchmark_config.enable_register,
    );
    let mut clients = tcp_client
        .setup_clients(&benchmark_config)
        .await
        .map_err(|e| e.to_string())?;

    info!("等待连接...");
    tcp_client.wait_for_connections(&mut clients).await;
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

    // 启动发送消息的线程
    let tx = async_proc_output_tx.inner.lock().await.clone();

    tokio::spawn(async move {
        let task = task.clone();
        let handles = tcp_client
            .spawn_message(clients, &*task.lock().await, &benchmark_config)
            .await
            .unwrap();
        task.lock().await.task_handle = Some(handles);
    });

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
    Ok("开始发送消息...".to_string())
}

#[command]
pub async fn stop_task() -> Result<String, String> {
    if let Some(task) = TASK.get() {
        let mut task = task.lock().await;
        if let Some(handle) = task.task_handle.take() {
            info!("stop_task: handle.take()");
            for h in handle {
                h.abort();
            }
            task.status.store(false, Ordering::SeqCst);
        }
        sleep(Duration::from_secs(1)).await;
        if let Some(handle) = task.count_handle.take() {
            info!("stop_task: handle.take()");
            handle.abort();
            task.status.store(false, Ordering::SeqCst);
            return Ok("任务已中断".to_string());
        }
    }
    Err("没有正在运行的任务".to_string())
}

async fn reset_task(task: Arc<Mutex<Task>>) {
    let task = task.lock().await;
    task.status.store(true, Ordering::SeqCst);
    task.counter.store(0, Ordering::SeqCst);
}
