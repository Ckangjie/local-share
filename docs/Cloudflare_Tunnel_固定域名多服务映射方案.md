# Cloudflare Tunnel 固定域名多服务映射方案

## 一、方案概述

可以使用 **Cloudflare Tunnel + 固定域名**，将本地多个服务端口映射到公网。

整体结构：

```text
公网用户
   │
   ▼
Cloudflare
   │
   ▼
固定 Cloudflare Tunnel
   │
   ├── demo.xxx.com
   │       ↓
   │    127.0.0.1:5500
   │
   ├── vue.xxx.com
   │       ↓
   │    127.0.0.1:5173
   │
   ├── api.xxx.com
   │       ↓
   │    127.0.0.1:8080
   │
   └── admin.xxx.com
           ↓
        127.0.0.1:3000
```

一个 Tunnel 可以同时代理多个本地服务。

---

## 二、适用场景

- 本地开发项目分享
- 给客户预览页面
- 给同事演示项目
- 本地前端项目临时公网访问
- 本地 API 接口分享
- 多个项目同时对外提供访问
- 搭建自己的本地服务分享工具

例如本地同时运行：

```text
5500  → HTML 项目
5173  → Vue/Vite 项目
3000  → 管理后台
8080  → Node/Bun API
```

可以通过一个 Tunnel 全部暴露出去。

---

## 三、固定域名方案

假设已经拥有：

```text
xxx.com
```

可以使用不同的子域名映射不同的本地端口：

| 公网地址 | 本地服务 | 端口 |
|---|---|---:|
| `https://demo.xxx.com` | 前端项目 A | 5500 |
| `https://vue.xxx.com` | Vue/Vite 项目 B | 5173 |
| `https://admin.xxx.com` | 管理后台 | 3000 |
| `https://api.xxx.com` | API 服务 | 8080 |

最终：

```text
demo.xxx.com  → 127.0.0.1:5500
vue.xxx.com   → 127.0.0.1:5173
admin.xxx.com → 127.0.0.1:3000
api.xxx.com   → 127.0.0.1:8080
```

---

## 四、创建 Tunnel

首先创建一个固定 Tunnel：

```bash
cloudflared tunnel create my-tunnel
```

创建完成后会获得 Tunnel ID 和 credentials 文件。

例如：

```text
Tunnel Name:
my-tunnel

Tunnel ID:
xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

credentials 文件通常位于：

```text
C:\Users\你的用户名\.cloudflared\
```

---

## 五、配置 config.yml

创建：

```text
C:\Users\你的用户名\.cloudflared\config.yml
```

示例：

```yaml
tunnel: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx

credentials-file: C:\Users\你的用户名\.cloudflared\xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.json

ingress:
  - hostname: demo.xxx.com
    service: http://127.0.0.1:5500

  - hostname: vue.xxx.com
    service: http://127.0.0.1:5173

  - hostname: admin.xxx.com
    service: http://127.0.0.1:3000

  - hostname: api.xxx.com
    service: http://127.0.0.1:8080

  - service: http_status:404
```

其中：

- `hostname`：公网访问地址
- `service`：本地实际服务

例如：

```yaml
- hostname: demo.xxx.com
  service: http://127.0.0.1:5500
```

表示：

```text
https://demo.xxx.com
        ↓
127.0.0.1:5500
```

---

## 六、配置 DNS

可以通过：

```bash
cloudflared tunnel route dns my-tunnel demo.xxx.com
cloudflared tunnel route dns my-tunnel vue.xxx.com
cloudflared tunnel route dns my-tunnel admin.xxx.com
cloudflared tunnel route dns my-tunnel api.xxx.com
```

为不同的子域名建立 Tunnel 路由。

---

## 七、启动 Tunnel

配置完成后：

```bash
cloudflared tunnel run my-tunnel
```

Tunnel 启动后，多个服务都会同时生效。

例如：

```text
https://demo.xxx.com
https://vue.xxx.com
https://admin.xxx.com
https://api.xxx.com
```

分别对应不同的本地端口。

---

## 八、一个 Tunnel 映射多个服务

整体结构：

```text
                         Cloudflare
                              │
                              ▼
                     ┌────────────────┐
                     │   my-tunnel    │
                     └───────┬────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
          ▼                  ▼                  ▼
       :5500               :5173              :8080
          │                  │                  │
          ▼                  ▼                  ▼
      项目 A              项目 B              API
```

因此：

> 不需要一个端口创建一个 Tunnel。

一个 Tunnel 就可以管理多个本地服务。

---

## 九、推荐使用子域名区分服务

例如：

```text
project1.xxx.com → 5500
project2.xxx.com → 5501
project3.xxx.com → 5502
project4.xxx.com → 5503
api.xxx.com      → 8080
admin.xxx.com    → 3000
```

对于本地项目分享，推荐使用子域名方式，因为配置简单、项目之间相互独立。

---

## 十、也可以使用路径区分

理论上也可以设计成：

```text
https://xxx.com/project-a
https://xxx.com/project-b
https://xxx.com/api
```

然后根据路径转发：

```text
xxx.com/project-a → 5500
xxx.com/project-b → 5501
xxx.com/api       → 8080
```

但是这种方式会涉及更多问题，例如：

- Vite `base`
- 静态资源路径
- 前端路由
- WebSocket
- API 请求地址

因此对于本地项目分享而言，更推荐使用子域名。

---

## 十一、推荐的最终结构

```text
项目 1
project1.xxx.com
        ↓
127.0.0.1:5500


项目 2
project2.xxx.com
        ↓
127.0.0.1:5501


项目 3
project3.xxx.com
        ↓
127.0.0.1:5502


Vue 项目
vue.xxx.com
        ↓
127.0.0.1:5173


后台
admin.xxx.com
        ↓
127.0.0.1:3000


API
api.xxx.com
        ↓
127.0.0.1:8080
```

---

## 十二、进一步做成本地分享工具

如果希望把 Cloudflare Tunnel 做成一个自己的工具，可以实现：

```text
┌────────────────────────────────────┐
│          Local Share               │
├────────────────────────────────────┤
│                                    │
│  检测到的本地服务                  │
│                                    │
│  ● 5173  Vite                      │
│  ● 5500  Live Server               │
│  ● 8080  Node API                  │
│  ● 3000  Vue Admin                 │
│  ● 7001  Vite                      │
│                                    │
│  选择端口：                        │
│  [ 5500 ▼ ]                        │
│                                    │
│  分享域名：                        │
│  demo.xxx.com                      │
│                                    │
│          [ 开始分享 ]              │
│                                    │
└────────────────────────────────────┘
```

工具自动完成：

```text
检测本地端口
      ↓
选择需要分享的服务
      ↓
确定对应子域名
      ↓
修改 / 生成 Tunnel 配置
      ↓
启动 cloudflared
      ↓
公网访问
```

---

## 十三、动态端口映射

例如本地启动：

```bash
npm run dev
```

Vite 使用：

```text
5173
```

工具检测到：

```text
5173
```

用户点击：

```text
[分享]
```

工具自动配置：

```text
dev.xxx.com → 127.0.0.1:5173
```

如果下次端口变成：

```text
5174
```

工具可以自动更新：

```text
dev.xxx.com → 127.0.0.1:5174
```

这样用户就不需要手动输入：

```bash
cloudflared tunnel route dns ...
cloudflared tunnel run ...
```

---

## 十四、最终架构

```text
                         ┌──────────────────┐
                         │     用户浏览器    │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │    Cloudflare    │
                         │   DNS / Tunnel   │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │   固定 Tunnel    │
                         │    my-tunnel     │
                         └────────┬─────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
        127.0.0.1:5500     127.0.0.1:5173     127.0.0.1:8080
              │                   │                   │
              ▼                   ▼                   ▼
           项目 A              项目 B                API
```

---

## 十五、核心结论

**固定域名 + 固定 Cloudflare Tunnel + 多 Ingress 规则**，可以实现一个 Tunnel 映射多个本地服务端口。

推荐方案：

```text
一个 Tunnel
    +
多个子域名
    +
多个本地端口
```

例如：

```text
demo.xxx.com  → 5500
vue.xxx.com   → 5173
admin.xxx.com → 3000
api.xxx.com   → 8080
```

对于「自动检测本地端口 + 选择端口 + 一键公网分享」工具，这个架构可以作为基础方案。
