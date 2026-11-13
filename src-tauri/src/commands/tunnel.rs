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
) -> Result<String, String> {
    let binary_path = resolve_cloudflared_path().ok_or_else(|| {
        "未检测到穿透引擎，请先完成引擎初始化下载。".to_string()
    })?;

    // 终止可能已经存在的同端口旧进程
    state.kill_process_by_port(port).await;

    let local_target = format!("http://127.0.0.1:{}", port);
    let host_header = format!("localhost:{}", port);
    let mut cmd = Command::new(&binary_path);
    cmd.args([
        "tunnel",
        "--url",
        &local_target,
        "--http-host-header",
        &host_header,
        "--no-autoupdate",
    ]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 cloudflared Quick Tunnel 失败: {}", e))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法捕获 cloudflared 日志流".to_string())?;

    let mut reader = BufReader::new(stderr).lines();
    let re_quick = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com")
        .map_err(|e| format!("正则编译失败: {}", e))?;

    let timeout_duration = Duration::from_secs(25);
    let parse_task = async {
        let mut log_accumulator = Vec::new();
        while let Ok(Some(line)) = reader.next_line().await {
            log_accumulator.push(line.clone());
            if let Some(matched) = re_quick.find(&line) {
                return Ok(matched.as_str().to_string());
            }
        }
        Err(format!(
            "Quick Tunnel 启动失败。日志: {:?}",
            log_accumulator.into_iter().rev().take(3).collect::<Vec<_>>()
        ))
    };

    let public_url = match tokio::time::timeout(timeout_duration, parse_task).await {
        Ok(res) => res?,
        Err(_) => {
            let _ = child.kill().await;
            return Err("创建 Cloudflare Quick Tunnel 超时 (25s)，请检查网络连接。".to_string());
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

    let tunnels_clone = state.tunnels.clone();
    let app_handle_clone = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let mut lock = tunnels_clone.lock().await;
            if let Some(active) = lock.get_mut(&port) {
                if let Ok(Some(_)) = active.child.try_wait() {
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
pub async fn stop_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    port: u16,
) -> Result<(), String> {
    state.kill_process_by_port(port).await;
    let _ = app.emit("tunnel-closed", port);
    Ok(())
}

#[tauri::command]
pub async fn stop_all_tunnels(state: State<'_, AppState>) -> Result<(), String> {
    state.kill_all_processes().await;
    Ok(())
}
