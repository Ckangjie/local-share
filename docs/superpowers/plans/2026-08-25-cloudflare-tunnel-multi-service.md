# Cloudflare Tunnel 单 Tunnel 动态 Ingress 多服务映射实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现单 Cloudflare Tunnel 守护进程 + 动态 Ingress 规则文件生成，支持本地多个服务端口同时映射到独立的固定子域名（如 `p5173.ccwu.cc`、`p8080.ccwu.cc`）。

**Architecture:** 前端维护主域名及 Tunnel 凭证（Credentials JSON）；Rust 端在用户分享端口时动态生成本地 `config.yml` 并维护单个 `cloudflared` 守护进程；同时完全保留原有的临时随机域名（Quick Tunnel）模式。

**Tech Stack:** Vue 3, Tauri 2 (Rust), Tokio, SCSS, JavaScript.

**Spec:** [`docs/superpowers/specs/2026-08-25-cloudflare-tunnel-multi-service-design.md`](file:///d:/demo/empty/LocalShare/docs/superpowers/specs/2026-08-25-cloudflare-tunnel-multi-service-design.md)

## Global Constraints
- 遵循 Vue 3 `<script setup>` 与项目原有 SCSS 变量与风格。
- 布尔变量以 `is/has/can/should` 命名。
- 异步操作配合 `try...catch`。
- 不引入不必要的新依赖。

---

### Task 1: 前端设置状态与凭据表单升级

**Files:**
- Modify: `src/hooks/useSettings.js`
- Modify: `src/components/SettingsModal.vue`

**Interfaces:**
- Consumes: `settings` 响应式对象
- Produces: `settings.value.customConfig`: `{ tunnelId, credentialsJson, baseDomain, subdomainPattern }`

- [ ] **Step 1: 更新 `useSettings.js` 默认值与存储读取逻辑**
  - 增加 `customConfig` 数据字段（`tunnelId`, `credentialsJson`, `baseDomain`, `subdomainPattern`），向下兼容历史配置。
- [ ] **Step 2: 更新 `SettingsModal.vue` 表单与自动解析**
  - 在固定域名模式下，提供主域名、Tunnel ID、凭据 JSON 内容输入框。
  - 支持粘贴 Credentials JSON 时自动提取 `TunnelID`。
- [ ] **Step 3: 语法与前端交互自检**

---

### Task 2: Rust 端 Ingress 动态生成与守护进程生命周期管理

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/commands/tunnel.rs`

**Interfaces:**
- Consumes: `start_tunnel(port, token, custom_domain, custom_config)`
- Produces: 动态 Ingress YAML 生成、`cloudflared tunnel --config ... run` 单守护进程调度

- [ ] **Step 1: 扩展 `AppState` (`src-tauri/src/state.rs`)**
  - 增加 `custom_daemon` 状态管理结构体，追踪运行中的端口列表、守护进程 Child 及临时配置文件路径。
  - 完善 `kill_process_by_port` 与 `kill_all_processes`，支持 Custom 守护进程的平滑重载与清理。
- [ ] **Step 2: 实现 Ingress YAML 与 Credentials 文件动态写入 (`src-tauri/src/commands/tunnel.rs`)**
  - 根据活跃端口列表动态组装 Ingress 规则，包含 `p<port>.<domain>` 映射与 404 兜底。
  - 启动/重载 `cloudflared` 守护进程并捕获 `stderr` 握手状态。
- [ ] **Step 3: 执行 `cargo check` 确保编译通过**

---

### Task 3: 前端穿透调度对接与多端口卡片展示

**Files:**
- Modify: `src/hooks/useTunnel.js`

**Interfaces:**
- Consumes: `useSettings`, Tauri `start_tunnel` / `stop_tunnel`
- Produces: `activeTunnels` 结构中的多端口独立二级域名 URL

- [ ] **Step 1: 更新 `useTunnel.js` 中的参数组装与调用**
  - 在 `startShare` 中读取 `customConfig` 并传递给 Rust 端。
  - 获取生成的 `https://p<port>.<baseDomain>` 并更新对应端口卡片状态。
- [ ] **Step 2: 保持 Quick 模式与错误通知体验一致**

---

### Task 4: 全流程语法检查与验证

**Files:**
- Test/Verify: `src-tauri/src/commands/tunnel.rs`, `src/hooks/useTunnel.js`, `src/components/SettingsModal.vue`

- [ ] **Step 1: 运行 Rust 语法检查 `cargo check --manifest-path src-tauri/Cargo.toml`**
- [ ] **Step 2: 检查前端代码语法与变量命名规范**
