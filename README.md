<p align="center"><img src="logo.png" width="128" alt="Su!"></p>

<h1 align="center">Su! — LAN File Transfer</h1>

<p align="center">
  <a href="README_zh.md"><img src="https://img.shields.io/badge/中文-README-red?style=for-the-badge" alt="中文"></a>
</p>

<p align="center"><em>咻一下，就到了！ Whoosh, it's there!</em></p>

<p align="center">
  <img src="screenshot.png" width="600" alt="Screenshot">
</p>

<p align="center">
  <a href="https://github.com/Hunter-Lies/Su/releases"><img src="https://img.shields.io/github/v/release/Hunter-Lies/Su?style=flat-square" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/Hunter-Lies/Su?style=flat-square" alt="MIT"></a>
</p>

---

A lightweight LAN file transfer tool. **No cloud, no login, no app install** — scan a QR code and transfer files between your phone and PC in seconds.

## Features

- **QR Share** — Right-click any file, generate a QR code, phone scans to download
- **Phone → PC** — Send files from your phone browser over LAN
- **Batch Download** — Select all, single, or batch with clear file list
- **5 Languages** — 简体中文 · 繁體中文 · English · 日本語 · 한국어
- **Dark Mode** — Classic & frosted glass themes, auto system follow
- **Smart Batching** — Auto device detection (iPhone / Android / Windows / Mac / Linux), group by batch
- **Sound + Popup** — Audio alerts and card notifications on receive
- **System Tray** — Minimize to tray, auto-start with system
- **Range Download** — Resumable chunked downloads for large files

## Quick Start

1. Download the [latest Release](https://github.com/Hunter-Lies/Su/releases)
2. Run `Su-v1.2.0-windows-x64.exe` (or the x86 version)
3. Drag files to the window → phone scans QR to download
4. Phone scans QR → select files to send to PC

## Tech Stack

| Layer | Stack |
|-------|-------|
| Desktop App | [Tauri v2](https://tauri.app/) + Rust |
| UI | Vanilla JS (ES Modules) + CSS Custom Properties |
| Mobile Pages | Plain HTML/JS/CSS, served by embedded HTTP server |
| HTTP Server | [tiny_http](https://github.com/tiny-http/tiny-http) |
| QR Code | [qrcode](https://crates.io/crates/qrcode) |
| Audio | [rodio](https://github.com/RustAudio/rodio) + [symphonia](https://github.com/pdeljanov/Symphonia) |

## Build

**Prerequisites**

| Tool | Version |
|------|---------|
| [Node.js](https://nodejs.org/) | >= 18 |
| [Rust](https://rustup.rs/) | >= 1.70 |
| WebView2 | Built-in on Windows 10/11 |

```bash
npm install
npm run tauri dev                    # Development
npm run tauri build                  # Release (x64)
npm run tauri build -- --target i686-pc-windows-msvc  # Release (x86)
```

Output in `src-tauri/target/`.

## Project Structure

```
Su/
├── src/                          # 桌面前端 · Desktop Frontend
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
├── src-tauri/                    # Rust 后端 · Rust Backend
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   ├── build.rs                  # 构建脚本
│   ├── capabilities/
│   │   └── default.json          # Tauri 权限
│   ├── icons/                    # 应用图标
│   ├── sounds/                   # 提示音文件 (mp3)
│   ├── web/                      # 手机端页面 · Mobile Pages
│   │   ├── send.html             # 发送页面
│   │   ├── download.html         # 单文件下载
│   │   ├── bundle_multi.html     # 多文件下载
│   │   ├── i18n.js               # 手机端多语言
│   │   └── fonts/                # Font Awesome（手机端）
│   ├── src/                      # Rust 源码 · Rust Source
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
├── README.md                     # English
├── README_zh.md                  # 中文
└── LICENSE                       # MIT
```

## Contributors

[![Contributors](https://contrib.rocks/image?repo=Hunter-Lies/Su)](https://github.com/Hunter-Lies/Su/graphs/contributors)

## License

MIT — see [LICENSE](LICENSE)

## Author

**HunterLies** · [Bilibili](https://space.bilibili.com/488494586) · [Website](https://htovo.com)
