# ⚡ LocalShare

> **轻量级本地 Web 服务一键公网分享桌面工具**  
> 基于 **Vue 3 + Vite + Tauri 2 (Rust)** 构建，打包体积仅 **3~5 MB**，启动毫秒级，无缝将本地开发服务一键映射至公网。

---

## ✨ 核心特性

- 🔍 **智能端口嗅探与服务识别**  
  自动扫描本机处于监听状态的 Web 开发端口（Vite、Webpack、Next.js、Nuxt、Live Server 等），通过 HTTP 探活精准过滤纯 JSON API 与系统底层 RPC，并自动打标技术栈特征。

- 🌐 **双穿透模式灵活切换**
  - **🎲 临时随机域名模式（Quick Tunnel）**：免账号、免域名、零配置，点击即可秒级生成 `https://*.trycloudflare.com` 临时访问链接。
  - **🔒 固定域名 Token 模式（Cloudflare Zero Trust）**：支持填入自定义 Tunnel Token，将本地服务永久稳定映射至您的专属个性化域名（例如 `https://du1.ccwu.cc`）。

- 🚀 **多端口并发分享**  
  支持同时对多个本地独立服务开启公网穿透，各隧道互不干扰；提供一键复制、浏览器快捷打开与顶部一键“全部停止”。

- 📦 **极简瘦身与按需引擎就绪**  
  安装包彻底剥离重型内置二进制，仅包含几兆的核心桌面底座；首次使用时通过流式进度弹窗按需就绪穿透引擎，并持久化缓存至本地。

- 🛡️ **进程生命周期安全守护**  
  底层维护精细的子进程映射池，当窗口关闭或应用退出时，自动级联清理所有后台 `cloudflared` 进程，杜绝孤儿进程与系统资源残留。

---

## 🛠️ 技术架构

```
LocalShare
├── 前端界面 (src/)
│   ├── Vue 3 (Composition API / <script setup>)
│   ├── Vite 5 + SCSS
│   └── 响应式工作台卡片流 + 模式设置弹窗
└── 桌面底座 (src-tauri/)
    ├── Tauri 2.0 (Rust)
    ├── 本地 TCP 端口扫描与 Web 探活引擎
    ├── 多隧道进程生命周期管理池
    └── 穿透引擎流式断点续传与缓存定位
```

---

## 🚀 快速起步

### 🛠️ 打包与开发环境要求

#### 1. 前端与运行环境
- **[Node.js](https://nodejs.org/)**：`>= 18.0.0`（推荐 LTS 20+）
- **包管理器**：**[pnpm](https://pnpm.io/)**（推荐 `>= 8.0`）

#### 2. Rust 编译底座（Tauri 2）
- **[Rust 工具链](https://www.rust-lang.org/)**：`rustc` / `cargo` (`>= 1.77.0`)
  ```bash
  # 安装 Rust (Windows / macOS / Linux)
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

#### 3. 各操作系统专属依赖
- **Windows（推荐打包目标）**：
  - **C++ 编译工具链**：安装 [Visual Studio 2022 生成工具](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，并勾选 **「使用 C++ 的桌面开发」**（包含 MSVC 编译器及 Windows 10/11 SDK）；
  - **WebView2 运行时**：Windows 10/11 默认自带（若缺失可安装 Evergreen Bootstrapper）。
- **macOS**：
  - 安装 Xcode 命令行工具：`xcode-select --install`
- **Linux (Ubuntu / Debian)**：
  ```bash
  sudo apt-get update
  sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

---

### 📦 开发与打包步骤

#### 1. 克隆项目与安装依赖
```bash
git clone git@github.com:Ckangjie/local-share.git
cd local-share
pnpm install
```

#### 2. 本地开发调试
```bash
# 仅启动前端开发服务器预览
pnpm dev

# 启动完整 Tauri 桌面端热重载环境
pnpm tauri dev
```

#### 3. 生产构建打包
```bash
# 一键编译并自动输出至根目录 LocalShare.exe (体积仅 3~5 MB)
pnpm tauri:build
```

---

## 📖 使用指南

### 方式一：临时快速分享（零门槛）
1. 启动 `LocalShare.exe`。
2. 软件会自动嗅探本机正在运行的前端开发服务（或在底部手动输入端口）。
3. 保持右上角为“随机”模式，点击目标服务的 **“开启分享”** 按钮。
4. 复制生成的 `https://*.trycloudflare.com` 链接即可发给同事或在手机端预览。

### 方式二：固定域名分享（品牌专属域名）
1. 在 [Cloudflare Zero Trust](https://one.dash.cloudflare.com/) 中创建一条 Tunnel 并获取专属 Token。
2. 在 Cloudflare 路由中添加您的域名映射（如 `du1.ccwu.cc` $\rightarrow$ `http://localhost:5173`）。
3. 打开 LocalShare 右上角 **⚙️ 设置**，切换为 **固定域名模式**，粘贴 Token 并保存。
4. 点击开启分享，即可直接通过 `https://du1.ccwu.cc` 稳定访问！

---

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 协议开源。
