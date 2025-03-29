// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use svroll::{
    context, model::{
        db_com, tauri_com::{self}, Rs2JsEntity
    }, AsyncProcInputTx
};
use tauri::{AppHandle, Emitter};
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
            tauri_com::receive_file,
            tauri_com::start_task,
            tauri_com::stop_task,
            tauri_com::process_client_file,
            tauri_com::write_file,
            tauri_com::load_config,
            tauri_com::get_clients,
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

fn rs2js<R: tauri::Runtime>(message: Rs2JsEntity, manager: &AppHandle<R>) {
    info!(?message, "rs2js");
    let payload = serde_json::to_string(&message).unwrap();
    manager.emit("rs2js", payload).unwrap();
}
