use std::process::Stdio;
use std::time::Duration;
use regex::Regex;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::commands::engine::resolve_cloudflared_path;
use crate::state::{ActiveTunnel, AppState};

#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    port: u16,
    token: Option<String>,
    custom_domain: Option<String>,
) -> Result<String, String> {
    // 若该端口已有正在运行的隧道，先优雅停止
    state.kill_process_by_port(port).await;

    let binary_path = resolve_cloudflared_path().ok_or_else(|| {
        "未检测到穿透引擎，请先完成引擎初始化下载。".to_string()
    })?;

    let local_target = format!("http://127.0.0.1:{}", port);
    let trimmed_token = token.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let is_custom_mode = trimmed_token.is_some();

    let mut cmd = Command::new(binary_path);
    if let Some(tok) = trimmed_token {
        cmd.args(["tunnel", "run", "--token", tok]);
    } else {
        cmd.args(["tunnel", "--url", &local_target, "--no-autoupdate"]);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    {
        // Windows 无黑框静默运行 (CREATE_NO_WINDOW)
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 cloudflared 进程失败: {}", e))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法捕获 cloudflared 日志流".to_string())?;

    let mut reader = BufReader::new(stderr).lines();
    let re_quick = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com")
        .map_err(|e| format!("正则编译失败: {}", e))?;
    let re_custom = Regex::new(r"(?i)Registered tunnel connection|Connection [a-f0-9-]+ registered|connIndex=\d+")
        .map_err(|e| format!("正则编译失败: {}", e))?;

    let timeout_duration = Duration::from_secs(25);
    let parse_task = async {
        let mut log_accumulator = Vec::new();
        while let Ok(Some(line)) = reader.next_line().await {
            log_accumulator.push(line.clone());
            if is_custom_mode {
                if re_custom.is_match(&line) {
                    let mut domain = custom_domain
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or("du1.ccwu.cc")
                        .to_string();
                    if domain.is_empty() {
                        domain = "du1.ccwu.cc".to_string();
                    }
                    if !domain.starts_with("http://") && !domain.starts_with("https://") {
                        domain = format!("https://{}", domain);
                    }
                    return Ok(domain);
                }
            } else if let Some(matched) = re_quick.find(&line) {
                return Ok(matched.as_str().to_string());
            }
        }
        Err(format!(
            "cloudflared 未能成功建立隧道。最后日志: {:?}",
            log_accumulator.into_iter().rev().take(3).collect::<Vec<_>>()
        ))
    };

    let public_url = match tokio::time::timeout(timeout_duration, parse_task).await {
        Ok(res) => res?,
        Err(_) => {
            let _ = child.kill().await;
            return Err(if is_custom_mode {
                "建立固定域名 Cloudflare 隧道超时 (25s)，请检查 Token 与网络连接。".to_string()
            } else {
                "创建 Cloudflare Quick Tunnel 超时 (25s)，请检查网络连接后重试。".to_string()
            });
        }
    };

    let active_tunnel = ActiveTunnel {
        child,
        port,
        public_url: public_url.clone(),
    };

    {
        let mut lock = state.tunnels.lock().await;
        lock.insert(port, active_tunnel);
    }

    // 后台任务：持续监听该端口子进程退出事件并通知前端
    let tunnels_clone = state.tunnels.clone();
    let app_handle_clone = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let mut lock = tunnels_clone.lock().await;
            if let Some(active) = lock.get_mut(&port) {
                if let Ok(Some(_exit_status)) = active.child.try_wait() {
                    // 进程已退出
                    lock.remove(&port);
                    let _ = app_handle_clone.emit("tunnel-closed", port);
                    break;
                }
            } else {
                break;
            }
        }
    });

    Ok(public_url)
}

#[tauri::command]
pub async fn stop_tunnel(state: State<'_, AppState>, port: u16) -> Result<(), String> {
    state.kill_process_by_port(port).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_all_tunnels(state: State<'_, AppState>) -> Result<(), String> {
    state.kill_all_processes().await;
    Ok(())
}
