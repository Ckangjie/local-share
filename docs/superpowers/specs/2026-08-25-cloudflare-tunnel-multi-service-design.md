# Cloudflare Tunnel 单 Tunnel 动态 Ingress 多服务映射技术方案

## 1. 背景与目标

当前 LocalShare 项目在固定域名模式下依赖云端 Token 单端口转发，无法在客户端动态同时为多个本地服务分配独立的固定子域名。
本项目将按照 `docs/Cloudflare_Tunnel_固定域名多服务映射方案.md` 的架构设计，实现**单 Tunnel 守护进程 + 本地动态 Ingress 规则文件生成 + 多端口子域名动态分配**。

### 核心收益
1. **多端口并发固定域名分享**：用户勾选/分享端口（如 5173、8080）时，自动生成 `https://p5173.ccwu.cc`、`https://p8080.ccwu.cc`。
2. **单实例资源高效**：由一个 `cloudflared` 守护进程统一承载多个端口的 Ingress 流量。
3. **保留免配置 Quick 模式**：普通用户或临时调试仍可无缝使用 `*.trycloudflare.com` 临时随机域名。

---

## 2. 架构设计

### 2.1 整体数据流

```text
[用户界面 Vue 3]
       │ 点击端口分享 (如 5173)
       ▼
[useTunnel.js]
       │ 读取 settings.customConfig (TunnelID, Credentials, BaseDomain)
       │ 调用 Tauri Command: start_tunnel({ port, ... })
       ▼
[Rust commands/tunnel.rs]
       │
       ├── Quick 模式：
       │     启动独立的 `cloudflared tunnel --url http://127.0.0.1:<port>`
       │
       └── Custom 模式 (动态 Ingress)：
             1. 将端口加入 `custom_active_ports` 集合
             2. 动态生成/更新应用数据目录下的 `config.yml`
             3. 若守护进程未运行，启动 `cloudflared tunnel --config config.yml run <Tunnel-ID>`
             4. 若守护进程已在运行，平滑重载守护进程
             5. 返回对应端口的公网地址 `https://p<port>.<BaseDomain>`
```

---

## 3. 详细设计

### 3.1 配置模型设计 (`src/hooks/useSettings.js`)

配置存储在 `localStorage` (`localshare_tunnel_settings`) 中：

```javascript
{
  mode: 'custom', // 'quick' | 'custom'
  // Quick 模式无须额外配置
  // Custom 模式配置结构：
  customConfig: {
    tunnelId: '',          // Cloudflare Tunnel UUID (如 8a1b2c3d-...)
    credentialsJson: '',   // 凭据 JSON 内容 (包含 AccountTag, TunnelSecret, TunnelID)
    baseDomain: 'ccwu.cc', // 主域名或泛域名 (如 ccwu.cc)
    subdomainPattern: 'p{port}' // 子域前缀模板，默认为 p{port}
  }
}
```

### 3.2 动态 Ingress 配置文件规范

Rust 端在运行时于应用配置目录（如 `%APPDATA%/LocalShare/` 或本地缓存路径）写入：

1. `credentials.json`：
```json
{
  "AccountTag": "...",
  "TunnelSecret": "...",
  "TunnelID": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
}
```

2. `config.yml`：
```yaml
tunnel: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
credentials-file: C:\path\to\LocalShare\credentials.json
protocol: http2
edge-ip-version: 4

ingress:
  - hostname: p5173.ccwu.cc
    service: http://127.0.0.1:5173
  - hostname: p8080.ccwu.cc
    service: http://127.0.0.1:8080
  - service: http_status:404
```

### 3.3 后端状态与进程管理 (`src-tauri/src/state.rs` & `src-tauri/src/commands/tunnel.rs`)

`AppState` 维护：
- `quick_tunnels: Arc<Mutex<HashMap<u16, ActiveTunnel>>>`：管理 Quick 模式的独立子进程；
- `custom_daemon: Arc<Mutex<Option<CustomDaemonState>>>`：
  - `child: Option<Child>`
  - `active_ports: HashMap<u16, String>`（端口与对应的公共 URL 映射）
  - `config_path: PathBuf`
  - `credentials_path: PathBuf`

生命周期行为：
- **`start_tunnel` (Custom 模式)**：
  1. 更新 `active_ports`；
  2. 覆写 `config.yml`；
  3. 启停/重启 `cloudflared` 守护进程；
  4. 捕获 `stderr` 的连接建立成功日志（超时 25 秒保护）；
  5. 返回格式化好的 `https://p<port>.<baseDomain>`。
- **`stop_tunnel` (Custom 模式)**：
  1. 从 `active_ports` 中移除对应 `port`；
  2. 若仍有活跃端口，重写 `config.yml` 并刷新守护进程；
  3. 若无剩余活跃端口，彻底杀死守护进程并清理。
- **`stop_all_tunnels`**：
  1. 终止所有 Quick 模式子进程；
  2. 终止 Custom 模式守护进程；
  3. 清空活跃状态映射。

### 3.4 前端交互与表单升级 (`src/components/SettingsModal.vue`)

- **Tab 1: 固定域名模式 (推荐)**：
  - 输入项：
    - 主域名 / 根域名（例如 `ccwu.cc`）；
    - Tunnel ID（支持自动从 JSON 提取）；
    - 凭据文件内容（支持直接粘贴 `.json` 文本）；
  - 操作指引提示：提示在 Cloudflare DNS 中添加泛解析 `CNAME *.ccwu.cc -> <Tunnel-ID>.cfargotunnel.com`。
- **Tab 2: 临时随机域名模式 (Quick)**：
  - 免配置，提示每次生成 `*.trycloudflare.com` 临时地址。

---

## 4. 异常处理与边缘情况

1. **凭证不合法**：前端在保存时先校验 JSON 格式及必需字段；后端在启动前校验 `credentials.json` 完整性。
2. **无端口运行时自动休眠**：最后一个端口停止分享时，守护进程自动回收，不空耗系统资源。
3. **应用退出级联销毁**：在 Tauri `on_window_event` / 退出钩子中调用 `kill_all_processes`，杜绝后台孤儿进程。
4. **网络协议强化**：全局采用 `--protocol http2` 和 `--edge-ip-version 4`，保证跨网连接稳定性。

---

## 5. 验证与测试计划

1. **单端口固定域名分享测试**：配置凭据后分享 `5173`，验证 `https://p5173.ccwu.cc` 能否成功生成并访问；
2. **多端口并发分享测试**：在 `5173` 运行状态下，追加分享 `8080`，验证两者各自对应的二级域名均可正常独立访问；
3. **单端口停止与全部停止测试**：停止 `5173` 时，验证 `8080` 依然保持在线；点击全部停止时，验证守护进程彻底退出；
4. **Quick 模式回归测试**：切换到临时域名模式，验证 `*.trycloudflare.com` 仍然正常工作。
