# ⚡ LocalShare

> **轻量级本地 Web 服务一键公网分享桌面工具**  
> 基于 **Vue 3 + Vite + Tauri 2 (Rust)** 构建，打包体积仅 **3~8 MB**，启动毫秒级，无缝将本地开发服务一键映射至公网。

---

## ✨ 核心特性

- ⚡ **主动输入端口一键分享**  
  免账号、免域名、零繁琐配置。主界面提供常驻端口输入栏，输入本地服务端口（如 `5173`、`3000`、`8080`）点击即可秒级生成 `https://*.trycloudflare.com` 公网临时访问链接。

- 🛡️ **Host 头自动重写（全面兼容 Vite 5/6）**  
  底层穿透引擎自动将转发请求的 Host 头重写为 `localhost:<port>`，彻底解决 Vite 5.4+ / Vite 6、Webpack 等因 DNS Rebinding 机制导致的 `403 Blocked request. This host is not allowed` 拦截，**被分享项目无需修改任何配置文件**。

- 🚀 **多端口并发分享**  
  支持同时对多个本地独立服务开启公网穿透，各隧道互不干扰；提供实时运行计时、一键复制、浏览器快捷打开与顶部一键“全部停止”。

- 📦 **极简瘦身与按需引擎就绪**  
  安装包彻底剥离重型内置二进制，仅包含核心桌面底座；首次使用时通过流式进度弹窗按需就绪穿透引擎，并持久化缓存至本地。

- 🔒 **进程生命周期安全守护**  
  底层维护精细的子进程映射池，当窗口关闭或应用退出时，自动级联清理所有后台 `cloudflared` 进程，杜绝孤儿进程与系统资源残留。

---

## 🛠️ 技术架构

```
LocalShare
├── 前端界面 (src/)
│   ├── Vue 3 (Composition API / <script setup>)
│   ├── Vite 5 + SCSS
│   └── 常驻端口输入栏 + 活跃分享卡片流
└── 桌面底座 (src-tauri/)
    ├── Tauri 2.0 (Rust)
    ├── 多隧道并发进程生命周期管理池
    ├── 请求 Host 自动重写与反向代理
    └── 穿透引擎流式断点续传与持久缓存
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
# 一键编译并自动输出至根目录 LocalShare.exe
pnpm tauri:build
```

---

## 📖 使用指南

1. 启动 `LocalShare.exe`。
2. 在顶部输入框中输入本地开发服务的端口号（例如 `5173`、`3000`、`8080`）。
3. 点击「⚡ 开始分享」或按回车，约 3~5 秒即可秒级生成 `https://*.trycloudflare.com` 临时公网地址。
4. 点击「复制地址」即可发给同事或在移动端浏览器访问，点击「浏览器打开」快捷跳转。
5. 随时点击卡片上的「停止」或顶栏「全部停止」一键终止分享。

---

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 协议开源。
