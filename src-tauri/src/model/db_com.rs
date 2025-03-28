use anyhow::Result;
use tauri::command;

use crate::context;

use super::{connect_param::ConnectParam, database::HistoryConfig};

#[command]
pub async fn get_history_config() -> Result<Vec<HistoryConfig>, String> {
    let db = context::get_database().await;
    let db_lock = db.lock().await;
    // 返回所有配置记录，如果没有记录，则返回自定义错误
    db_lock
        .get_configs()
        .await
        .map_err(|_| "没有找到配置记录".to_string())
}

#[command]
pub async fn load_history_config(id: i64) -> Result<ConnectParam, String> {
    let db = context::get_database().await;
    let db_lock = db.lock().await;

    let history_config = db_lock.get_config(id).await.unwrap();
    let config = match history_config {
        Some(config) => config,
        None => return Err("没有找到配置记录".into()),
    };

    let connect_param: ConnectParam =
        serde_json::from_value(config.data).map_err(|_| "解析配置记录失败".to_string())?;
    Ok(connect_param)
}

#[command]
pub async fn clear_history_config() -> Result<bool, String> {
    let db = context::get_database().await;
    let db_lock = db.lock().await;
    db_lock.delete_all_configs().await.map_err(|_| "清除配置记录失败".to_string())
}