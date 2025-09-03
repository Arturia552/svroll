use anyhow::Result;
use tauri::command;

use crate::context;

use super::{connect_param::ConnectParam, database::HistoryConfig};

/// 获取所有历史配置
/// 
/// 用于前端显示历史配置列表
/// 
/// # 返回
/// 成功返回历史配置记录列表，失败返回错误信息
#[command]
pub async fn get_history_config() -> Result<Vec<HistoryConfig>, String> {
    let db = context::get_database();
    // 使用读锁访问数据库
    let db_read = db.read().await;
    db_read
        .get_configs()
        .await
        .map_err(|_| "没有找到配置记录".to_string())
}

/// 加载特定的历史配置
/// 
/// 通过ID获取并转换为连接参数
/// 
/// # 参数
/// * `id` - 配置记录ID
/// 
/// # 返回
/// 成功返回连接参数，失败返回错误信息
#[command]
pub async fn load_history_config(id: i64) -> Result<ConnectParam, String> {
    let db = context::get_database();
    // 使用读锁访问数据库
    let db_read = db.read().await;

    let history_config = db_read.get_config(id).await.unwrap();
    let config = match history_config {
        Some(config) => config,
        None => return Err("没有找到配置记录".into()),
    };

    let connect_param: ConnectParam =
        serde_json::from_value(config.data).map_err(|_| "解析配置记录失败".to_string())?;
    Ok(connect_param)
}

/// 清除所有历史配置
/// 
/// 删除数据库中的所有配置记录
/// 
/// # 返回
/// 成功返回true，失败返回错误信息
#[command]
pub async fn clear_history_config() -> Result<bool, String> {
    let db = context::get_database();
    // 使用读锁访问数据库 - 即使是删除操作，数据库内部也会处理锁
    let db_read = db.read().await;
    db_read.delete_all_configs().await.map_err(|_| "清除配置记录失败".to_string())
}