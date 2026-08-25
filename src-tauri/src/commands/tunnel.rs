use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use regex::Regex;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::commands::engine::resolve_cloudflared_path;
use crate::state::{ActiveTunnel, AppState, CustomDaemon, CustomTunnelConfig};

static GATEWAY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// 启动本地 17890 多端口智能代理网关
fn ensure_gateway_running() {
    if GATEWAY_INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:17890").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("绑定本地网关端口 17890 失败: {}", e);
                return;
            }
        };

        loop {
            if let Ok((mut client_stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};

                    let mut buf = vec![0u8; 16384];
                    let n = match client_stream.read(&mut buf).await {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };

                    let raw_slice = &buf[..n];
                    let header_str = String::from_utf8_lossy(raw_slice);

                    // 从 Host 或 X-Forwarded-Host 提取目标端口
                    let mut target_port: Option<u16> = None;
                    for line in header_str.lines() {
                        let lower = line.to_ascii_lowercase();
                        if lower.starts_with("host:") || lower.starts_with("x-forwarded-host:") {
                            let colon_pos = line.find(':').unwrap_or(0);
                            let host_val = line[colon_pos + 1..].trim();
                            if let Some(pos) = host_val.find('.') {
                                let prefix = &host_val[..pos];
                                let clean = prefix.trim_start_matches('p').trim_start_matches('P');
                                if let Ok(p) = clean.parse::<u16>() {
                                    target_port = Some(p);
                                    break;
                                }
                            }
                        }
                    }

                    let port = match target_port {
                        Some(p) => p,
                        None => return,
                    };

                    if let Ok(mut target_stream) = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                        let is_ws = header_str.to_ascii_lowercase().contains("upgrade: websocket");
                        if is_ws {
                            if target_stream.write_all(raw_slice).await.is_ok() {
                                let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut target_stream).await;
                            }
                        } else {
                            // 查找 Header 结束标志
                            let header_end_pos = raw_slice.windows(4).position(|w| w == b"\r\n\r\n");
                            let (headers_raw, body_raw) = match header_end_pos {
                                Some(pos) => (&raw_slice[..pos], &raw_slice[pos + 4..]),
                                None => (raw_slice, &[][..]),
                            };

                            let h_str = String::from_utf8_lossy(headers_raw);
                            let mut modified_lines = Vec::new();
                            let mut has_connection = false;

                            for line in h_str.lines() {
                                let lower = line.to_ascii_lowercase();
                                if lower.starts_with("host:") {
                                    modified_lines.push(format!("Host: 127.0.0.1:{}", port));
                                } else if lower.starts_with("connection:") {
                                    modified_lines.push("Connection: close".to_string());
                                    has_connection = true;
                                } else {
                                    modified_lines.push(line.to_string());
                                }
                            }
                            if !has_connection {
                                modified_lines.push("Connection: close".to_string());
                            }

                            let mut payload = modified_lines.join("\r\n").into_bytes();
                            payload.extend_from_slice(b"\r\n\r\n");
                            payload.extend_from_slice(body_raw);

                            if target_stream.write_all(&payload).await.is_ok() {
                                let _ = tokio::io::copy(&mut target_stream, &mut client_stream).await;
                            }
                        }
                    }
                });
            }
        }
    });
}

/// 计算指定端口在 Custom Ingress 模式下的公网访问 URL
fn compute_custom_public_url(port: u16, base_domain: &str, subdomain_pattern: &str) -> String {
    let clean_domain = base_domain.trim().trim_start_matches("http://").trim_start_matches("https://").trim_end_matches('/');
    let final_domain = if clean_domain.is_empty() { "du1.ccwu.cc" } else { clean_domain };
    let prefix = if subdomain_pattern.is_empty() || subdomain_pattern == "p{port}" {
        format!("p{}", port)
    } else {
        subdomain_pattern.replace("{port}", &port.to_string())
    };
    format!("https://{}.{}", prefix, final_domain)
}

#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    port: u16,
    token: Option<String>,
    custom_domain: Option<String>,
    custom_config: Option<CustomTunnelConfig>,
) -> Result<String, String> {
    let binary_path = resolve_cloudflared_path().ok_or_else(|| {
        "未检测到穿透引擎，请先完成引擎初始化下载。".to_string()
    })?;

    // 启动本地智能分发网关
    ensure_gateway_running();

    // 提取有效 Token（来自参数、custom_config.token 或 credentials_json 解析）
    let mut effective_token = token
        .as_deref()
        .or_else(|| custom_config.as_ref().and_then(|c| c.token.as_deref()))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);

    // 若未直接提供 Token，尝试从 credentials_json 中反解
    if effective_token.is_none() {
        if let Some(creds_str) = custom_config.as_ref().and_then(|c| c.credentials_json.as_deref()) {
            let trimmed = creds_str.trim();
            if trimmed.starts_with("eyJh") {
                effective_token = Some(trimmed.to_string());
            }
        }
    }

    let is_custom_mode = custom_config.is_some() || token.is_some();

    if is_custom_mode {
        let tok = effective_token.ok_or_else(|| {
            "当前处于固定域名模式，但未检测到有效的 Cloudflare Token，请先在设置中填入 Token。".to_string()
        })?;

        // ========== 模式 A: 固定域名 + 本地智能网关 (支持多端口并发) ==========
        let base_domain = custom_config
            .as_ref()
            .and_then(|c| c.base_domain.as_deref())
            .unwrap_or_else(|| custom_domain.as_deref().unwrap_or("du1.ccwu.cc"));
        let pattern = custom_config
            .as_ref()
            .and_then(|c| c.subdomain_pattern.as_deref())
            .unwrap_or("p{port}");

        let public_url = compute_custom_public_url(port, base_domain, pattern);

        let mut daemon_lock = state.custom_daemon.lock().await;

        // 若守护进程已经在运行且健康，直接将当前端口加入活跃列表返回
        if let Some(daemon) = daemon_lock.as_mut() {
            if let Ok(None) = daemon.child.try_wait() {
                daemon.active_ports.insert(port, public_url.clone());
                return Ok(public_url);
            }
        }

        // 启动新的 cloudflared tunnel run --token <tok>
        if let Some(mut old_daemon) = daemon_lock.take() {
            let _ = old_daemon.child.kill().await;
        }

        let mut cmd = Command::new(&binary_path);
        cmd.args(["tunnel", "run", "--token", &tok, "--protocol", "http2"]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(windows)]
        {
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
        let re_token = Regex::new(r"(?i)Registered tunnel connection|Connection [a-f0-9-]+ registered|connIndex=\d+|Starting tunnel")
            .map_err(|e| format!("正则编译失败: {}", e))?;

        let timeout_duration = Duration::from_secs(25);
        let parse_task = async {
            let mut log_acc = Vec::new();
            while let Ok(Some(line)) = reader.next_line().await {
                log_acc.push(line.clone());
                if re_token.is_match(&line) {
                    return Ok(());
                }
            }
            Err(format!(
                "cloudflared 未能成功建立固定隧道。日志: {:?}",
                log_acc.into_iter().rev().take(3).collect::<Vec<_>>()
            ))
        };

        if let Err(_) = tokio::time::timeout(timeout_duration, parse_task).await {
            let _ = child.kill().await;
            return Err("建立固定域名 Cloudflare 隧道超时 (25s)，请检查 Token 与网络连接。".to_string());
        }

        let mut active_ports = HashMap::new();
        active_ports.insert(port, public_url.clone());

        *daemon_lock = Some(CustomDaemon {
            child,
            active_ports,
            config_path: PathBuf::new(),
            credentials_path: PathBuf::new(),
        });

        // 启动后台监听守护进程退出任务
        let daemon_arc = state.custom_daemon.clone();
        let app_handle_clone = app.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                let mut lock = daemon_arc.lock().await;
                if let Some(daemon) = lock.as_mut() {
                    if let Ok(Some(_)) = daemon.child.try_wait() {
                        let ports: Vec<u16> = daemon.active_ports.keys().copied().collect();
                        *lock = None;
                        for p in ports {
                            let _ = app_handle_clone.emit("tunnel-closed", p);
                        }
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        return Ok(public_url);
    }

    // ========== 模式 B: 临时随机域名 Quick Tunnel 模式 ==========
    state.kill_quick_process_by_port(port).await;

    let local_target = format!("http://127.0.0.1:{}", port);
    let mut cmd = Command::new(&binary_path);
    cmd.args([
        "tunnel",
        "--url",
        &local_target,
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
        let mut lock = state.quick_tunnels.lock().await;
        lock.insert(port, active_tunnel);
    }

    let tunnels_clone = state.quick_tunnels.clone();
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
    // 1. 检查并停止 Quick 模式
    state.kill_quick_process_by_port(port).await;

    // 2. 检查并从 Custom Ingress 守护进程中移除该端口
    let mut daemon_lock = state.custom_daemon.lock().await;
    if let Some(mut daemon) = daemon_lock.take() {
        daemon.active_ports.remove(&port);

        if daemon.active_ports.is_empty() {
            // 已无活跃端口，彻底终止守护进程
            let _ = daemon.child.kill().await;
        } else {
            *daemon_lock = Some(daemon);
        }
    }

    let _ = app.emit("tunnel-closed", port);
    Ok(())
}

#[tauri::command]
pub async fn stop_all_tunnels(state: State<'_, AppState>) -> Result<(), String> {
    state.kill_all_processes().await;
    Ok(())
}

