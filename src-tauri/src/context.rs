use std::sync::Arc;

use anyhow::Result;
use once_cell::sync::OnceCell;

use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

use crate::{model::database::Database, state::AppState};

// 全局应用状态
static APP_STATE: OnceCell<Arc<AppState>> = OnceCell::new();

/// 初始化应用状态
pub async fn init_app_state(app_handle: &AppHandle) -> Result<()> {
    // 初始化数据库
    let app_dir = app_handle.path().app_data_dir()?;

    // 确保目录存在
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    let database = Database::new(&app_handle).await?;

    // 创建并存储应用状态
    let state = AppState::new(database);
    APP_STATE.get_or_init(|| Arc::new(state));

    Ok(())
}

/// 获取应用状态引用
pub fn get_app_state() -> &'static Arc<AppState> {
    APP_STATE.get().expect("应用状态尚未初始化")
}

/// 获取数据库引用
pub async fn get_database() -> Arc<RwLock<Database>> {
    get_app_state().database().clone()
}
