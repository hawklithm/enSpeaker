// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
use audio::{start_recording, stop_recording};
use enspeaker_lib::init_logger;

fn main() {
    if let Err(e) = init_logger() {
        eprintln!("日志初始化失败: {}", e);
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
        ]).plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
