use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

/// 获取持久化引擎缓存目录 (%LOCALAPPDATA%\LocalShare\bin)
pub fn get_engine_cache_dir() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("LocalShare").join("bin")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile).join(".localshare").join("bin")
    } else {
        PathBuf::from("bin")
    }
}

/// 定位可用的 cloudflared 可执行文件路径
pub fn resolve_cloudflared_path() -> Option<PathBuf> {
    // 1. 检查环境变量 PATH
    if let Ok(path) = which::which("cloudflared") {
        return Some(path);
    }
    if let Ok(path) = which::which("cloudflared.exe") {
        return Some(path);
    }

    // 2. 检查本地 AppData 缓存目录
    let cache_binary = get_engine_cache_dir().join("cloudflared.exe");
    if cache_binary.exists() {
        return Some(cache_binary);
    }

    // 3. 检查开发工作目录相对路径
    let dev_candidates = [
        PathBuf::from("src-tauri/binaries/cloudflared.exe"),
        PathBuf::from("binaries/cloudflared.exe"),
        PathBuf::from("cloudflared.exe"),
    ];
    for dev_path in dev_candidates {
        if dev_path.exists() {
            return Some(dev_path);
        }
    }

    None
}

#[tauri::command]
pub async fn check_engine_status() -> Result<bool, String> {
    Ok(resolve_cloudflared_path().is_some())
}

#[tauri::command]
pub async fn download_engine(app: AppHandle) -> Result<String, String> {
    let cache_dir = get_engine_cache_dir();
    fs::create_dir_all(&cache_dir).map_err(|e| format!("创建引擎缓存目录失败: {}", e))?;

    let target_file = cache_dir.join("cloudflared.exe");
    let temp_file = cache_dir.join("cloudflared.exe.tmp");

    // 高速镜像源与官方 Release 源
    let urls = [
        "https://ghfast.top/https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe",
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe",
    ];

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("创建网络客户端失败: {}", e))?;

    let mut last_error = String::new();

    for url in urls {
        match client.get(url).send().await {
            Ok(res) if res.status().is_success() => {
                let total_size = res.content_length().unwrap_or(0);
                let mut stream = res.bytes_stream();
                let mut file = File::create(&temp_file)
                    .map_err(|e| format!("创建临时文件失败: {}", e))?;

                let mut downloaded: u64 = 0;
                let mut last_emitted_percent: f64 = 0.0;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result.map_err(|e| format!("读取下载数据流失败: {}", e))?;
                    file.write_all(&chunk)
                        .map_err(|e| format!("写入文件失败: {}", e))?;

                    downloaded += chunk.len() as u64;

                    if total_size > 0 {
                        let percent = (downloaded as f64 / total_size as f64 * 100.0).round();
                        if percent - last_emitted_percent >= 1.0 || downloaded == total_size {
                            last_emitted_percent = percent;
                            let _ = app.emit(
                                "engine-download-progress",
                                DownloadProgress {
                                    downloaded,
                                    total: total_size,
                                    percent,
                                },
                            );
                        }
                    }
                }

                file.flush().map_err(|e| format!("刷新文件缓存失败: {}", e))?;
                drop(file);

                if target_file.exists() {
                    let _ = fs::remove_file(&target_file);
                }
                fs::rename(&temp_file, &target_file)
                    .map_err(|e| format!("重命名引擎文件失败: {}", e))?;

                return Ok(target_file.to_string_lossy().to_string());
            }
            Ok(res) => {
                last_error = format!("下载请求返回异常状态码: {}", res.status());
            }
            Err(e) => {
                last_error = format!("连接下载源失败: {}", e);
            }
        }
    }

    if temp_file.exists() {
        let _ = fs::remove_file(temp_file);
    }

    Err(format!("下载穿透引擎失败: {}", last_error))
}
