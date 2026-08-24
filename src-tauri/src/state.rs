use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub port: u16,
    pub public_url: String,
}

pub struct ActiveTunnel {
    pub child: Child,
    pub port: u16,
    pub public_url: String,
}

#[derive(Default)]
pub struct AppState {
    pub tunnels: Arc<Mutex<HashMap<u16, ActiveTunnel>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 强制终止指定端口的子进程
    pub async fn kill_process_by_port(&self, port: u16) {
        let mut lock = self.tunnels.lock().await;
        if let Some(mut active) = lock.remove(&port) {
            let _ = active.child.kill().await;
        }
    }

    /// 强制终止所有正在运行的子进程（退出时级联销毁）
    pub async fn kill_all_processes(&self) {
        let mut lock = self.tunnels.lock().await;
        for (_, mut active) in lock.drain() {
            let _ = active.child.kill().await;
        }
    }
}
