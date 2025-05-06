use serde::Serialize;
use std::fs;
use std::env;
// use std::path::Path; // <--- 移除未使用的导入
use tauri::Manager;
use log::{info, error}; // 添加 log 导入

mod baidu_uploader; // <--- 添加模块声明
mod baidu_userinfo; // <--- 添加模块声明
use baidu_uploader::BaiduUploader; // <--- 使用模块
use baidu_userinfo::{BaiduUserInfo, UserInfoResponse, QuotaResponse}; // <--- Import structs

// 定义返回给前端的数据结构
#[derive(Serialize, Debug)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    readonly: bool,
}

// 定义 Tauri 命令
#[tauri::command]
fn list_directory(path: String) -> Result<Vec<FileInfo>, String> {
    println!("Rust: Received path: {}", path);
    let mut entries = Vec::new();
    let read_dir = match fs::read_dir(&path) {
        Ok(dir) => dir,
        Err(e) => {
            let error_msg = format!("无法读取目录 '{}': {}", path, e);
            eprintln!("{}", error_msg);
            return Err(error_msg);
        }
    };

    for entry_result in read_dir {
        match entry_result {
            Ok(entry) => {
                let path_buf = entry.path();
                let metadata = match entry.metadata() {
                    Ok(meta) => meta,
                    Err(e) => {
                        eprintln!("无法获取元数据 {:?}: {}", path_buf, e);
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                let path_str = path_buf.to_string_lossy().into_owned();
                let is_dir = metadata.is_dir();
                let size = if is_dir { None } else { Some(metadata.len()) };
                let readonly = metadata.permissions().readonly();

                entries.push(FileInfo {
                    name,
                    path: path_str,
                    is_dir,
                    size,
                    readonly,
                });
            }
            Err(e) => {
                eprintln!("读取目录条目时出错: {}", e);
            }
        }
    }
    Ok(entries)
}

#[tauri::command]
fn get_initial_path(app: tauri::AppHandle) -> Result<String, String> {
    // 1. 尝试获取可执行文件所在的目录
    let exe_dir_path = env::current_exe()
        .ok() // Option<PathBuf>
        .and_then(|p| p.parent().map(|p| p.to_path_buf())); // Option<PathBuf>

    let initial_path_result = match exe_dir_path {
        Some(dir) => Ok(dir), // 如果成功获取 exe 目录，使用它
        None => {
            // 2. 如果获取 exe 目录失败，回退到尝试 home 目录
            println!("无法获取 exe 目录，尝试 home 目录...");
            app.path().home_dir()
                // 3. 如果 home 目录也失败，回退到 app data 目录
                .or_else(|_| {
                    println!("无法获取 home 目录，尝试 app data 目录...");
                    app.path().app_data_dir()
                })
        }
    };

    // 处理最终结果
    let path = initial_path_result
        .map_err(|e| format!("无法获取初始路径: {}", e))?
        .to_string_lossy()
        .into_owned();

    println!("Rust: Determined initial path: {}", path);
    Ok(path)
}

#[tauri::command]
async fn upload_file_to_baidupan(local_path: String, remote_dir: String, access_token: String, _app_handle: tauri::AppHandle) -> Result<String, String> {
    info!("Attempting to upload: {} to remote dir: {} using provided token", local_path, remote_dir);

    // 直接使用从前端传递过来的 access_token
    if access_token.is_empty() {
        error!("Access Token 为空，无法上传");
        return Err("Access Token 为空，请在设置中配置".to_string());
    }

    let uploader = BaiduUploader::new(access_token);

    // 确保 remote_dir 是一个有效的目录路径，例如 "/apps/myapp" 或 "/来自：mcp_server"
    // 这里我们直接使用用户提供的值，但实际应用中可能需要验证
    let default_remote_dir = "/来自：rust_file_manager_uploads"; // 默认上传目录
    let target_remote_dir = if remote_dir.is_empty() { default_remote_dir } else { &remote_dir };

    match uploader.upload_file(&local_path, target_remote_dir).await {
        Ok(response_value) => {
            info!("文件上传成功: {:?}", response_value);
            // 将 serde_json::Value 转换为字符串返回
            serde_json::to_string(&response_value)
                .map_err(|e| format!("序列化上传响应失败: {}", e))
        }
        Err(e) => {
            error!("文件上传失败: {}", e);
            Err(format!("文件上传失败: {}", e))
        }
    }
}

// --- 新增命令：获取百度用户信息和配额 ---
#[tauri::command]
async fn get_baidu_user_info(access_token: String) -> Result<UserInfoResponse, String> {
    if access_token.is_empty() {
        error!("Access Token is empty when trying to get user info");
        return Err("Access Token is empty. Please configure it in settings.".to_string());
    }
    info!("Fetching Baidu user info...");
    let user_info_fetcher = BaiduUserInfo::new(access_token);
    user_info_fetcher.get_user_info().await
        .map_err(|e| {
            let err_msg = format!("Failed to get user info: {}", e);
            error!("{}", err_msg);
            err_msg
        })
}

#[tauri::command]
async fn get_baidu_quota(access_token: String) -> Result<QuotaResponse, String> {
     if access_token.is_empty() {
        error!("Access Token is empty when trying to get quota");
        return Err("Access Token is empty. Please configure it in settings.".to_string());
    }
    info!("Fetching Baidu quota info...");
    let user_info_fetcher = BaiduUserInfo::new(access_token);
    // Pass None for optional checkexpire/checkfree for now
    user_info_fetcher.get_quota(None, None).await
         .map_err(|e| {
             let err_msg = format!("Failed to get quota info: {}", e);
             error!("{}", err_msg);
             err_msg
         })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        let log_plugin = tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info);

        #[cfg(debug_assertions)]
        let log_plugin = log_plugin.with_colors(tauri_plugin_log::fern::colors::ColoredLevelConfig::default());
        
        app.handle().plugin(log_plugin.build())?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        list_directory,
        get_initial_path,
        upload_file_to_baidupan,
        get_baidu_user_info,
        get_baidu_quota
    ])
    .run(tauri::generate_context!("tauri.conf.json"))
    .expect("error while running tauri application");
}
