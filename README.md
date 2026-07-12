<p align="center"><img src="logo.png" width="128" alt="Su!"></p>

<h1 align="center">Su! — 局域网文件快传</h1>

<p align="center">
  <a href="README_en.md"><img src="https://img.shields.io/badge/English-README-blue?style=for-the-badge" alt="English"></a>
</p>

<p align="center"><em>咻一下，就到了！</em></p>

<p align="center">
  <img src="screenshot.png" width="600" alt="截图">
</p>

<p align="center">
  <a href="https://github.com/Hunter-Lies/Su-Share/releases/latest"><img src="https://img.shields.io/github/v/release/Hunter-Lies/Su-Share?style=flat-square" alt="Release"></a>
  <a href="https://github.com/Hunter-Lies/Su-Share/releases/latest"><img src="https://img.shields.io/github/downloads/Hunter-Lies/Su/total?style=flat-square" alt="Downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/Hunter-Lies/Su-Share?style=flat-square" alt="MIT"></a>
  <a href="https://github.com/Hunter-Lies/Su-Share/stargazers"><img src="https://img.shields.io/github/stars/Hunter-Lies/Su-Share?style=flat-square" alt="Stars"></a>
</p>

---

**Su!** 是一款轻量级局域网文件传输工具。无需云端、无需登录、无需安装 App——手机扫个码，文件就传到电脑上了。反过来，右键任意文件生成二维码，手机一扫就下走了。

## 目录结构

```
Su-Share/
├── package.json
├── logo.png
├── screenshot.png
├── src/
│   ├── index.html                    # 桌面端主界面
│   ├── pages/                        # 动态加载的页面
│   │   ├── share.html                # 文件分享页
│   │   ├── received.html             # 接收记录页
│   │   ├── settings.html             # 设置主页
│   │   ├── settings-software.html    # 软件设置
│   │   ├── settings-security.html    # 安全与隐私
│   │   ├── settings-receive.html     # 接收设置
│   │   ├── settings-notification.html# 通知设置
│   │   └── settings-appearance.html  # 外观设置
│   ├── js/
│   │   ├── main.js                   # 入口
│   │   ├── settings.js               # 设置页逻辑
│   │   ├── share.js                  # 分享逻辑
│   │   ├── received.js               # 接收逻辑
│   │   ├── theme.js                  # 主题
│   │   ├── state.js                  # 状态和DOM引用
│   │   ├── utils.js                  # 工具函数
│   │   └── i18n/
│   │       ├── index.js
│   │       ├── zh-CN.js / zh-TW.js / en.js / ja.js / ko.js
│   ├── css/
│   │   └── styles.css
│   └── assets/
│       └── fonts/                    # Font Awesome 本地字体
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                   # Rust入口
│   │   ├── lib.rs                    # App构建和配置
│   │   ├── commands.rs               # Tauri命令
│   │   ├── http.rs                   # HTTP服务器（处理手机连接）
│   │   ├── state.rs                  # 应用状态
│   │   ├── sound.rs                  # 通知声音
│   │   ├── qr.rs                     # 二维码生成
│   │   ├── utils.rs                  # 工具
│   │   └── com_shellext.rs           # 右键菜单注册（Windows）
│   ├── web/                          # 手机端HTML页面
│   │   ├── send.html                 # 发送页面
│   │   ├── download.html             # 单文件下载页
│   │   ├── bundle_multi.html         # 多文件下载页
│   │   ├── auth.html / pickup.html   # 彩蛋
│   │   ├── i18n.js                   # 手机端多语言
│   │   └── fonts/                    # 手机端字体
│   ├── Cargo.toml
│   └── tauri.conf.json
├── README.md
├── README_en.md
├── CHANGELOG.md
└── LICENSE
```


## 下载

| 版本 | 下载 | 适用 |
|------|------|------|
| 64 位 | [Su-v1.2.1-windows-x64.zip](https://github.com/Hunter-Lies/Su-Share/releases/latest/download/Su-v1.2.1-windows-x64.zip) | 主流 Windows 10/11 |
| 32 位 | [Su-v1.2.1-windows-x86.zip](https://github.com/Hunter-Lies/Su-Share/releases/latest/download/Su-v1.2.1-windows-x86.zip) | 老旧 / 32 位系统 |

> 解压后双击 `su.exe` 即可运行，**无需安装**。如遇 SmartScreen 拦截，点击「更多信息」→「仍要运行」。
>
> macOS / Linux 用户可自行构建（见下方构建说明），部分功能（右键菜单、毛玻璃）暂不可用。

## 功能特性

- **二维码分享** — 右键任意文件 → 生成二维码 → 手机扫码即下载
- **手机传电脑** — 手机浏览器打开网页，选择文件发送到电脑
- **多文件批量** — 全选 / 单选 / 批量下载，文件清单清晰
- **断点续传** — Range 分块下载，传一半关了浏览器也能续上
- **智能批次** — 自动识别设备 (iPhone / Android / Windows / Mac / Linux)，按批次分组显示
- **5 种语言** — 简体中文 · 繁體中文 · English · 日本語 · 한국어，手机端自动跟随
- **深色模式** — 经典 / 毛玻璃主题，自动跟随系统
- **提示音 + 弹窗** — 收到文件时播放音效、弹出卡片通知
- **系统托盘** — 关闭到托盘、开机自启
- **绿色免安装** — 一个 exe，解压即用，不写注册表

## 快速开始

1. [下载](https://github.com/Hunter-Lies/Su-Share/releases/latest)对应版本 zip，解压
2. 双击 `su.exe`，软件启动后窗口显示二维码
3. **手机发电脑**：手机扫码 → 选择文件发送
4. **电脑发手机**：拖文件到窗口 → 手机扫码下载

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | [Tauri v2](https://tauri.app/) + Rust |
| 前端 UI | Vanilla JS (ES Modules) + CSS 自定义属性 |
| 手机端页面 | 纯 HTML/JS/CSS，由内置 HTTP 服务提供 |
| HTTP 服务 | [tiny_http](https://github.com/tiny-http/tiny-http) |
| 二维码 | [qrcode](https://crates.io/crates/qrcode) |
| 音频 | [rodio](https://github.com/RustAudio/rodio) + [symphonia](https://github.com/pdeljanov/Symphonia) |

## 构建

**环境要求**

| 工具 | 版本 |
|------|------|
| [Node.js](https://nodejs.org/) | >= 18 |
| [Rust](https://rustup.rs/) | >= 1.70 |
| WebView2 | Windows 10/11 已内置 |

```bash
npm install
npm run tauri dev                         # 开发模式
npm run tauri build                       # 构建 64 位
npm run tauri build -- --target i686-pc-windows-msvc  # 构建 32 位
```

构建产物在 `src-tauri/target/release/` 下。

## 项目结构

```
Su/
├── src/                          # 桌面前端
│   ├── index.html                # 主窗口
│   ├── css/
│   │   └── styles.css            # 全部样式
│   ├── js/                       # JavaScript 模块
│   │   ├── main.js               # 入口
│   │   ├── i18n.js               # 多语言 (zh-CN/zh-TW/en/ja/ko)
│   │   ├── state.js              # DOM 引用与应用状态
│   │   ├── utils.js              # Toast、剪贴板、二维码工具
│   │   ├── settings.js           # 设置页面
│   │   ├── share.js              # 文件分享 UI
│   │   ├── received.js           # 接收记录 UI
│   │   └── theme.js              # 主题与外观
│   └── assets/
│       └── fonts/                # Font Awesome 6（本地）
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   ├── build.rs                  # 构建脚本
│   ├── capabilities/
│   │   └── default.json          # Tauri 权限
│   ├── icons/                    # 应用图标
│   ├── sounds/                   # 提示音文件 (mp3)
│   ├── web/                      # 手机端页面
│   │   ├── send.html             # 发送页面
│   │   ├── download.html         # 单文件下载
│   │   ├── bundle_multi.html     # 多文件下载
│   │   ├── i18n.js               # 手机端多语言
│   │   └── fonts/                # Font Awesome（手机端）
│   ├── src/                      # Rust 源码
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 应用初始化、Tauri 命令注册、托盘
│   │   ├── http.rs               # HTTP 服务器、路由、上传/下载处理
│   │   ├── commands.rs           # Tauri 命令（文件操作、设置、右键菜单）
│   │   ├── state.rs              # 应用状态与持久化（JSON）
│   │   ├── sound.rs              # 音效播放
│   │   ├── qr.rs                 # 二维码生成
│   │   ├── com_shellext.rs       # Windows 右键菜单扩展
│   │   └── utils.rs              # 工具函数（MIME、格式化、响应）
│   ├── AppxManifest.xml          # Windows 右键菜单注册清单
│   ├── register_sparse.ps1       # 右键菜单注册脚本
│   └── unregister_sparse.ps1     # 右键菜单卸载脚本
├── package.json                  # Node 依赖
├── logo.png                      # Logo
├── screenshot.png                # 截图
├── README.md                     # 中文说明（默认）
├── README_en.md                  # 英文说明
└── LICENSE                       # MIT 许可证
```

## 贡献者

[![Contributors](https://contrib.rocks/image?repo=Hunter-Lies/Su-Share)](https://github.com/Hunter-Lies/Su-Share/graphs/contributors)

欢迎提交 Issue 和 PR！

## 许可证

MIT — 详见 [LICENSE](LICENSE)

## 作者

**HunterLies** · [B站](https://space.bilibili.com/488494586) · [官网](https://htovo.com)
