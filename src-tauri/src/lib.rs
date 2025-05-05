use serde::Serialize;
use std::fs;
use std::env;
use std::path::Path;
// use std::path::PathBuf; // 移除未使用的导入
use tauri::Manager;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        list_directory,
        get_initial_path
    ])
    .run(tauri::generate_context!("tauri.conf.json"))
    .expect("error while running tauri application");
}
