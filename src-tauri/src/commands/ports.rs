use std::collections::HashSet;
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServiceInfo {
    pub port: u16,
    pub url: String,
    pub title: String,
    pub tag: String,
    pub tag_label: String,
}

/// 常见开发服务常用端口列表（兜底探测集合）
const COMMON_DEV_PORTS: &[u16] = &[
    5173, 5174, 5175, 5176, 5177, 5178, // Vite / Vue 3 / Svelte / React
    5500, 5501, 5502, 5503, 5504, 5505, // VS Code Live Server
    7000, 7001, 7002, 7003, 7004, 7005, // 微前端 / 业务系统
    8080, 8081, 8082, 8083, 8084, 8085, // Vue CLI / Webpack
    8000, 8001, 8002, 8003, 8004, 8005, // Umi / Django / Web
    3000, 3001, 3002, 3003, 3004, 3005, // React / Next.js / Nuxt
    9527, 9528, 9529,                   // Vue Element Admin / 后台脚手架
    9000, 9001, 9002,                   // Webpack Dev / 微前端
    5000, 5001, 5002,                   // Vite 备用
    4173, 4174,                         // Vite Preview
    1420,                               // Tauri Dev
];

/// 常见非 Web 服务系统与数据库端口黑名单
const EXCLUDED_PORTS: &[u16] = &[
    22, 53, 67, 68, 135, 137, 138, 139, 445, 1433, 1521, 3306, 3389, 5432, 5900, 6379, 11211,
    27017, 28017, 9092,
];

#[cfg(windows)]
fn get_listening_tcp_ports_windows() -> Vec<u16> {
    use std::ptr::null_mut;
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, TCP_TABLE_OWNER_PID_ALL,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;

    let mut ports = HashSet::new();

    unsafe {
        let mut size = 0u32;
        let _ = GetExtendedTcpTable(
            null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if size > 0 {
            let mut buffer = vec![0u8; size as usize];
            let ret = GetExtendedTcpTable(
                buffer.as_mut_ptr() as _,
                &mut size,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );

            if ret == 0 {
                let num_entries = *(buffer.as_ptr() as *const u32);
                let row_ptr = buffer.as_ptr().add(4);
                let row_size = 24usize;

                for i in 0..num_entries as usize {
                    let current_row = row_ptr.add(i * row_size);
                    let state = *(current_row as *const u32);
                    let raw_port = *(current_row.add(8) as *const u32);

                    if state == 2 {
                        let port = u16::from_be(raw_port as u16);
                        if port > 0 && !EXCLUDED_PORTS.contains(&port) {
                            ports.insert(port);
                        }
                    }
                }
            }
        }
    }

    for &p in COMMON_DEV_PORTS {
        ports.insert(p);
    }

    let mut list: Vec<u16> = ports.into_iter().collect();
    list.sort_unstable();
    list
}

#[cfg(not(windows))]
fn get_listening_tcp_ports_windows() -> Vec<u16> {
    COMMON_DEV_PORTS.to_vec()
}

/// 提取 HTML <title> 标签内容
fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    if let Some(start_idx) = lower.find("<title") {
        let after_tag = &html[start_idx..];
        if let Some(tag_close) = after_tag.find('>') {
            let content = &after_tag[tag_close + 1..];
            if let Some(end_idx) = content.to_lowercase().find("</title>") {
                let title = content[..end_idx].trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
    }
    None
}

/// 智能识别 Web 前端/UI 页面服务类型（过滤纯 Node API / 系统杂项端口）
fn identify_service_type(
    port: u16,
    html: &str,
    headers: &reqwest::header::HeaderMap,
    status_code: u16,
) -> Option<(String, String, String)> {
    if status_code >= 400 {
        return None;
    }

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    // 排除纯 JSON API 响应（未包含 HTML 渲染结构）
    if content_type.contains("application/json") && !html.contains("<html") {
        return None;
    }

    let lower_html = html.to_lowercase();
    let title = extract_html_title(html);

    // 1. VS Code Live Server 识别
    if lower_html.contains("live-server")
        || lower_html.contains("live server ws")
        || lower_html.contains("code injected by live-server")
        || (port >= 5500 && port <= 5510 && (title.is_some() || lower_html.contains("<html")))
    {
        return Some((
            title.unwrap_or_else(|| "Live Server 预览页面".to_string()),
            "live-server".to_string(),
            "VS Code Live".to_string(),
        ));
    }

    // 2. Vite / Vue 3 工程识别
    if lower_html.contains("@vite/client")
        || lower_html.contains("/@vite/")
        || lower_html.contains("vite plugin")
        || (port >= 5173 && port <= 5180 && (title.is_some() || lower_html.contains("<html")))
    {
        return Some((
            title.unwrap_or_else(|| "Vue / Vite 开发页面".to_string()),
            "vite".to_string(),
            "Vite / Vue".to_string(),
        ));
    }

    // 3. Webpack / Vue CLI / React 识别
    if lower_html.contains("webpackhotupdate")
        || lower_html.contains("webpack-dev-server")
        || lower_html.contains("sockjs-node")
        || (port >= 8080 && port <= 8090 && (title.is_some() || lower_html.contains("<html")))
    {
        return Some((
            title.unwrap_or_else(|| "React / Webpack 工程页面".to_string()),
            "webpack".to_string(),
            "React / Webpack".to_string(),
        ));
    }

    // 4. Next.js / Nuxt 识别
    if lower_html.contains("__next") || lower_html.contains("next.js") {
        return Some((
            title.unwrap_or_else(|| "Next.js 页面".to_string()),
            "next-nuxt".to_string(),
            "Next.js".to_string(),
        ));
    }
    if lower_html.contains("__nuxt") || lower_html.contains("nuxt") {
        return Some((
            title.unwrap_or_else(|| "Nuxt 页面".to_string()),
            "next-nuxt".to_string(),
            "Nuxt".to_string(),
        ));
    }

    // 5. 拥有明确 HTML 标题的本地 Web 页面
    if let Some(t) = title {
        return Some((t, "web".to_string(), "Web 页面".to_string()));
    }

    // 6. 包含标准 HTML 结构
    if lower_html.contains("<!doctype html")
        || (lower_html.contains("<html") && lower_html.contains("<body"))
    {
        return Some((
            format!("本地 Web 页面 (:{}", port),
            "web".to_string(),
            "Web 页面".to_string(),
        ));
    }

    // 其余纯 API、系统 RPC 或未知二进制服务均过滤排除
    None
}

/// 探测单个端口是否为 Web 服务
async fn probe_web_port(client: &Client, port: u16) -> Option<WebServiceInfo> {
    let url = format!("http://127.0.0.1:{}", port);

    let res = client
        .get(&url)
        .header("User-Agent", "LocalShare-Prober/1.0")
        .header("Accept", "text/html,application/xhtml+xml,application/json,*/*")
        .send()
        .await
        .ok()?;

    let status = res.status().as_u16();
    let headers = res.headers().clone();
    let body_text = res.text().await.unwrap_or_default();

    let (title, tag, tag_label) = identify_service_type(port, &body_text, &headers, status)?;

    Some(WebServiceInfo {
        port,
        url: format!("http://localhost:{}", port),
        title,
        tag,
        tag_label,
    })
}

#[tauri::command]
pub async fn scan_web_services() -> Result<Vec<WebServiceInfo>, String> {
    let candidate_ports = get_listening_tcp_ports_windows();
    if candidate_ports.is_empty() {
        return Ok(Vec::new());
    }

    let client = Client::builder()
        .timeout(Duration::from_millis(600))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("初始化网络探测客户端失败: {}", e))?;

    let mut tasks = Vec::new();
    for port in candidate_ports {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            probe_web_port(&client_clone, port).await
        }));
    }

    let mut detected_services = Vec::new();
    for task in tasks {
        if let Ok(Some(service)) = task.await {
            detected_services.push(service);
        }
    }

    // 排序
    detected_services.sort_by(|a, b| a.port.cmp(&b.port));

    Ok(detected_services)
}
