// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use svroll::{
    context, model::{
        db_com, task_com::{self}
    }, rs2js, AsyncProcInputTx
};
use tokio::sync::{mpsc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_output_tx),
        })
        .invoke_handler(tauri::generate_handler![
            task_com::receive_file,
            task_com::start_task,
            task_com::stop_task,
            task_com::process_client_file,
            task_com::write_file,
            task_com::load_config,
            task_com::get_clients,
            db_com::get_history_config,
            db_com::load_history_config,
            db_com::clear_history_config,
        ])
        .setup(|app| {
            let app_handle = app.handle().to_owned();

            let db_app_handle = app_handle.clone();
            tokio::spawn(async move {
                if let Err(e) = context::init_app_state(&db_app_handle).await {
                    tracing::error!("无法初始化数据库: {}", e);
                } else {
                    info!("数据库初始化成功");
                }
            });

            tokio::spawn(async move {
                loop {
                    if let Some(output) = async_proc_output_rx.recv().await {
                        rs2js(output, &app_handle);
                    }
                }
            });

            info!("Tauri app started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

