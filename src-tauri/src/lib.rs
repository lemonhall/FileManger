use serde::{Deserialize, Serialize};
use std::fs;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tauri::{AppHandle, Manager};
use log::{info, error, warn};
use std::time::UNIX_EPOCH;

mod baidu_uploader;
mod baidu_userinfo;
use baidu_uploader::BaiduUploader;
use baidu_userinfo::{BaiduUserInfo, UserInfoResponse, QuotaResponse};

const TIMESTAMPS_FILE_NAME: &str = "upload_timestamps.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimestampEntry {
    #[serde(rename = "lastUploaded")]
    last_uploaded: u64,
}

fn get_timestamps_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory at {:?}: {}", app_data_dir, e))?;
    }
    Ok(app_data_dir.join(TIMESTAMPS_FILE_NAME))
}

fn load_timestamps_from_file(app_handle: &AppHandle) -> Result<HashMap<String, TimestampEntry>, String> {
    let path = get_timestamps_file_path(app_handle)?;
    if !path.exists() {
        info!("Timestamps file not found at {:?}, returning empty map.", path);
        return Ok(HashMap::new());
    }

    let file = File::open(&path)
        .map_err(|e| format!("Failed to open timestamps file at {:?}: {}", path, e))?;
    let reader = BufReader::new(file);
    
    match serde_json::from_reader(reader) {
        Ok(map) => Ok(map),
        Err(e) if e.is_eof() => {
            warn!("Timestamps file at {:?} is empty, returning empty map.", path);
            Ok(HashMap::new())
        }
        Err(e) => {
            error!("Failed to deserialize timestamps from {:?}: {}", path, e);
            Err(format!("Failed to read or parse timestamps file: {}", e))
        }
    }
}

fn save_timestamps_to_file(app_handle: &AppHandle, timestamps: &HashMap<String, TimestampEntry>) -> Result<(), String> {
    let path = get_timestamps_file_path(app_handle)?;
    let temp_path = path.with_extension("json.tmp");

    let temp_file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temporary timestamps file at {:?}: {}", temp_path, e))?;
    let writer = BufWriter::new(temp_file);
    
    serde_json::to_writer_pretty(writer, timestamps)
        .map_err(|e| format!("Failed to serialize timestamps to temporary file {:?}: {}", temp_path, e))?;

    fs::rename(&temp_path, &path)
        .map_err(|e| format!("Failed to rename temporary timestamps file {:?} to {:?}: {}", temp_path, path, e))?;
    
    info!("Successfully saved timestamps to {:?}", path);
    Ok(())
}

#[tauri::command]
fn get_all_upload_timestamps(app_handle: AppHandle) -> Result<HashMap<String, u64>, String> {
    info!("Fetching all upload timestamps");
    match load_timestamps_from_file(&app_handle) {
        Ok(timestamp_entries) => {
            let result = timestamp_entries.into_iter()
                .map(|(path, entry)| (path, entry.last_uploaded))
                .collect();
            Ok(result)
        }
        Err(e) => {
            error!("Error in get_all_upload_timestamps: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
fn set_upload_timestamp(app_handle: AppHandle, file_path: String, timestamp: u64) -> Result<(), String> {
    info!("Setting upload timestamp for {}: {}", file_path, timestamp);
    let mut timestamps = load_timestamps_from_file(&app_handle)
        .map_err(|e| {
            error!("Failed to load timestamps in set_upload_timestamp for {}: {}", file_path, e);
            e
        })?;

    timestamps.insert(file_path.clone(), TimestampEntry { last_uploaded: timestamp });
    
    save_timestamps_to_file(&app_handle, &timestamps)
        .map_err(|e| {
            error!("Failed to save timestamps in set_upload_timestamp for {}: {}", file_path, e);
            e
        })
}

#[derive(Serialize, Debug)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    readonly: bool,
    modified: Option<u64>,
}

#[tauri::command]
fn list_directory(path: String) -> Result<Vec<FileInfo>, String> {
    info!("Rust: Received path for list_directory: {}", path);
    let mut entries = Vec::new();
    let read_dir = match fs::read_dir(&path) {
        Ok(dir) => dir,
        Err(e) => {
            let error_msg = format!("无法读取目录 '{}': {}", path, e);
            error!("{}", error_msg);
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
                        error!("无法获取元数据 {:?}: {}", path_buf, e);
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                let path_str = path_buf.to_string_lossy().into_owned();
                let is_dir = metadata.is_dir();
                let size = if is_dir { None } else { Some(metadata.len()) };
                let readonly = metadata.permissions().readonly();
                
                let modified_timestamp = metadata.modified()
                    .ok()
                    .and_then(|mod_time| mod_time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_millis() as u64);

                entries.push(FileInfo {
                    name,
                    path: path_str,
                    is_dir,
                    size,
                    readonly,
                    modified: modified_timestamp,
                });
            }
            Err(e) => {
                error!("读取目录条目时出错: {}", e);
            }
        }
    }
    Ok(entries)
}

#[tauri::command]
fn get_initial_path(app: tauri::AppHandle) -> Result<String, String> {
    let exe_dir_path = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let initial_path_result = match exe_dir_path {
        Some(dir) => Ok(dir),
        None => {
            warn!("无法获取 exe 目录，尝试 home 目录...");
            app.path().home_dir()
                .or_else(|_| {
                    warn!("无法获取 home 目录，尝试 app data 目录...");
                    app.path().app_data_dir()
                })
        }
    };

    let path = initial_path_result
        .map_err(|e| format!("无法获取初始路径: {}", e))?
        .to_string_lossy()
        .into_owned();

    info!("Rust: Determined initial path: {}", path);
    Ok(path)
}

#[tauri::command]
async fn upload_file_to_baidupan(local_path: String, remote_dir: String, access_token: String, _app_handle: tauri::AppHandle) -> Result<String, String> {
    info!("Attempting to upload: {} to remote dir: {} using provided token", local_path, remote_dir);

    if access_token.is_empty() {
        error!("Access Token 为空，无法上传");
        return Err("Access Token 为空，请在设置中配置".to_string());
    }

    let uploader = BaiduUploader::new(access_token);

    let default_remote_dir = "/来自：rust_file_manager_uploads";
    let target_remote_dir = if remote_dir.is_empty() { default_remote_dir } else { &remote_dir };

    match uploader.upload_file(&local_path, target_remote_dir).await {
        Ok(response_value) => {
            info!("文件上传成功: {:?}", response_value);
            serde_json::to_string(&response_value)
                .map_err(|e| format!("序列化上传响应失败: {}", e))
        }
        Err(e) => {
            error!("文件上传失败: {}", e);
            Err(format!("文件上传失败: {}", e))
        }
    }
}

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
            .level(if cfg!(debug_assertions) { log::LevelFilter::Info } else { log::LevelFilter::Warn });

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
        get_baidu_quota,
        get_all_upload_timestamps,
        set_upload_timestamp
    ])
    .run(tauri::generate_context!("tauri.conf.json"))
    .expect("error while running tauri application");
}
