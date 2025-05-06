use serde::Serialize;
use std::fs;
use std::env;
// use std::path::Path; // <--- 移除未使用的导入
use tauri::Manager;
use log::{info, error};

#[tauri::command]
fn get_initial_path(app: tauri::AppHandle) -> Result<String, String> {
    // ... existing code ...
    Ok(path)
}

#[tauri::command]
async fn upload_file_to_baidupan(local_path: String, remote_dir: String, access_token: String, _app_handle: tauri::AppHandle) -> Result<String, String> { // <--- _app_handle
    info!("Attempting to upload: {} to remote dir: {} using provided token", local_path, remote_dir);

    // 直接使用从前端传递过来的 access_token
    // ... existing code ...
} 