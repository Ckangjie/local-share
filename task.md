# LocalShare 极简轻量化改造任务清单（方案 B：Tauri 2 + 首次按需引擎下载）

> **目标**：将应用打包体积从 **73 MB** 彻底缩减至 **3 MB ~ 5 MB**，启动速度提升至毫秒级，内存占用降低 85%，同时完整保留多服务并发分享与退出自动清理守护能力。

---

## 阶段一：Tauri 2 瘦身配置与解耦 (`src-tauri/`)

- [x] **Task 1.1: 移除内置重量级二进制声明**
  - 清理 `src-tauri/tauri.conf.json` 中的 `externalBin: ["binaries/cloudflared"]`。
  - 调整窗口基础参数（宽度 `440px`，高度 `640px`，最小宽度 `400px`，最小高度 `520px`，支持自由缩放 `resizable: true`）。
- [x] **Task 1.2: 配置 Rust 编译优化选项 (`src-tauri/Cargo.toml`)**
  - 开启 Release 模式 `opt-level = 3` 与 `lto = "thin"`。
  - 开启 `panic = "abort"` 与 `strip = true` 自动剔除调试符号表，使核心 exe 压缩至极小体积。

---

## 阶段二：引擎按需下载与持久化缓存 (`src-tauri/src/commands/engine.rs`)

- [x] **Task 2.1: 本地缓存定位与可用性探测**
  - 定义本地标准存储路径：`%LOCALAPPDATA%\LocalShare\bin\cloudflared.exe`。
  - 探测优先级：环境变量 PATH > 本地 AppData 缓存目录 > 开发目录。
  - 导出 Tauri 命令：`check_engine_status() -> Result<bool, String>`。
- [x] **Task 2.2: 流式断点续传与下载进度广播**
  - 配置主备高速下载源（Cloudflare 官方 Release 源 + 国内 CDN 加速镜像源）。
  - 使用 `reqwest` 流式分块下载，并通过 Tauri Event `engine-download-progress` 向前端实时广播下载百分比与速度。
  - 写入临时文件校验完整性后原子重命名为 `cloudflared.exe`。
  - 导出 Tauri 命令：`download_engine() -> Result<String, String>`。

---

## 阶段三：多隧道并发进程池管理 (`src-tauri/src/commands/tunnel.rs`, `state.rs`)

- [x] **Task 3.1: Rust 全局状态多进程池改造 (`state.rs`)**
  - 将原本单实例的 `Option<ActiveTunnel>` 升级为 `Arc<Mutex<HashMap<u16, ActiveTunnel>>>` 映射池。
  - 实现 `kill_process_by_port(port)` 与 `kill_all_processes()` 级联销毁方法。
- [x] **Task 3.2: 多端口并发隧道启停 (`tunnel.rs`)**
  - `start_tunnel(port: u16)`：检查并就绪引擎，启动独立子进程，流式正则提取公网 URL，成功后加入映射池。
  - `stop_tunnel(port: u16)`：仅终止对应端口子进程，不影响其他并发隧道。
  - `stop_all_tunnels()`：一键清理所有运行中隧道。
  - 后台异步监听：单子进程异常退出时精确向前端广播 `tunnel-closed(port)`。
- [x] **Task 3.3: 窗口退出级联清理守护 (`main.rs`)**
  - 监听 `CloseRequested` 和 `Destroyed` 事件，退出前强制调用 `kill_all_processes()`，杜绝孤儿进程。

---

## 阶段四：智能 Web 探活与 API 过滤 (`src-tauri/src/commands/ports.rs`)

- [x] **Task 4.1: 严格过滤非前端 UI 端口**
  - 并发探活端口时，检查响应 `Content-Type` 与 HTML 内容。
  - 严格过滤纯 JSON API（如 Express / Nest / Koa 后端）、系统 RPC 与非 200 端口。
- [x] **Task 4.2: 前端技术栈特征提取与打标**
  - 识别并输出结构化特征：`tag`（`vite` / `webpack` / `live-server` / `next-nuxt` / `web`）与 `tagLabel`。

---

## 阶段五：前端下载交互与打包交付

- [x] **Task 5.1: 首次使用下载弹窗/进度条组件 (`src/components/EngineModal.vue`)**
  - 当首次点击分享且本地未就绪引擎时，弹出优雅的毛玻璃进度对话框，展示下载进度百分比。
  - 下载完成后自动无缝触发之前的分享操作。
- [x] **Task 5.2: 全量适配与编译测试**
  - 前端适配 Tauri 2 invoke 与多端口事件调用。
  - 成功编译输出仅 **8.5 MB** 的极简 `LocalShare.exe`。
