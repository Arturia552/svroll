use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};

use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

use crate::{model::database::Database, MqttClientData};

// 全局静态变量，用于存储客户端上下文
pub static MQTT_CLIENT_CONTEXT: Lazy<DashMap<String, MqttClientData>> = Lazy::new(DashMap::new);
pub static TCP_CLIENT_CONTEXT: Lazy<DashMap<String, OwnedWriteHalf>> = Lazy::new(DashMap::new);

// 数据库全局状态
pub static DATABASE: OnceCell<Mutex<Database>> = OnceCell::new();

// 数据库初始化状态
pub static DATABASE_INITIALIZED: OnceCell<tokio::sync::Notify> = OnceCell::new();

// 数据库初始化函数，确保只会初始化一次
pub async fn init_database(app_handle: &tauri::AppHandle) -> Result<(), anyhow::Error> {
    if DATABASE.get().is_some() {
        return Ok(());
    }

    let notify = DATABASE_INITIALIZED.get_or_init(|| tokio::sync::Notify::new());

    match Database::new(app_handle).await {
        Ok(db) => {
            DATABASE
                .set(Mutex::new(db))
                .map_err(|_| anyhow::anyhow!("数据库已被初始化"))?;

            notify.notify_waiters();
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("数据库初始化失败: {}", e)),
    }
}

pub async fn get_database() -> &'static Mutex<Database> {
    if DATABASE.get().is_none() {
        let notify = DATABASE_INITIALIZED.get().expect("数据库初始化状态未设置");
        notify.notified().await;
    }

    DATABASE.get().expect("数据库未初始化")
}
