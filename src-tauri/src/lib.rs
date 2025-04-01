// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use log::{info, warn, error};
use simplelog::*;
use std::fs::File;
use std::fs;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 logs 目录
    fs::create_dir_all("logs")?;

    
    // 配置日志
    CombinedLogger::init(vec![
        // 输出到文件
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("logs/app.log")?
        ),
        // 同时输出到控制台
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ])?;

    info!("日志系统初始化成功");
    Ok(())
}
