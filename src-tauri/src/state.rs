use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTunnelConfig {
    pub tunnel_id: Option<String>,
    pub credentials_json: Option<String>,
    pub base_domain: Option<String>,
    pub subdomain_pattern: Option<String>,
    pub token: Option<String>,
}

pub struct ActiveTunnel {
    pub child: Child,
    pub port: u16,
    pub public_url: String,
}

pub struct CustomDaemon {
    pub child: Child,
    pub active_ports: HashMap<u16, String>, // port -> public_url
    pub config_path: PathBuf,
    pub credentials_path: PathBuf,
}

#[derive(Default)]
pub struct AppState {
    pub quick_tunnels: Arc<Mutex<HashMap<u16, ActiveTunnel>>>,
    pub custom_daemon: Arc<Mutex<Option<CustomDaemon>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            quick_tunnels: Arc::new(Mutex::new(HashMap::new())),
            custom_daemon: Arc::new(Mutex::new(None)),
        }
    }

    /// 强制终止指定端口的 Quick 模式子进程
    pub async fn kill_quick_process_by_port(&self, port: u16) {
        let mut lock = self.quick_tunnels.lock().await;
        if let Some(mut active) = lock.remove(&port) {
            let _ = active.child.kill().await;
        }
    }

    /// 强制终止所有正在运行的子进程（退出时级联销毁）
    pub async fn kill_all_processes(&self) {
        // 1. 清理全部 Quick 模式子进程
        {
            let mut lock = self.quick_tunnels.lock().await;
            for (_, mut active) in lock.drain() {
                let _ = active.child.kill().await;
            }
        }

        // 2. 清理 Custom 模式守护进程
        {
            let mut daemon_lock = self.custom_daemon.lock().await;
            if let Some(mut daemon) = daemon_lock.take() {
                let _ = daemon.child.kill().await;
            }
        }
    }
}
