// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate paho_mqtt as mqtt;
use iot_connect_app::{
    model::tauri_com::{self, Rs2JsEntity},
    AsyncProcInputTx,
};
use tauri::Manager;
use tokio::sync::{mpsc, Mutex};
use tracing::info; 

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_output_tx),
        })
        .invoke_handler(tauri::generate_handler![
            tauri_com::receive_file,
            tauri_com::start_task,
            tauri_com::stop_task,
            tauri_com::process_client_file
        ])
        .setup(|app| {
            let app_handle = app.handle();
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

fn rs2js<R: tauri::Runtime>(message: Rs2JsEntity, manager: &impl Manager<R>) {
    info!(?message, "rs2js");
    let payload = serde_json::to_string(&message).unwrap();
    manager.emit_all("rs2js", payload).unwrap();
}
