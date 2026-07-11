<p align="center"><img src="logo.png" width="128" alt="Su!"></p>

# Su! - 局域网文件快传

> 咻一下，就到了！Whoosh, it's there!

轻量级局域网文件传输工具。手机扫码即可与电脑互传文件，无需云端，纯本地网络。

<p align="center"><img src="screenshot.png" width="600" alt="Su! 截图"></p>

## 功能特性

- **二维码分享** — 右键任意文件生成二维码，手机扫码即下载
- **手机传电脑** — 手机浏览器发送文件到电脑，同一局域网即可
- **多文件支持** — 批量下载，全选/单选，进度追踪
- **多语言** — 简体中文 / 繁體中文 / English / 日本語 / 한국어
- **深色模式** — 经典 / 毛玻璃主题，自动跟随系统
- **批次管理** — 智能分批，设备识别 (iPhone/Android/Windows/Mac/Linux)
- **提示音** — 收到文件时播放自定义音效
- **系统托盘** — 支持关闭到托盘、开机自启
- **断点续传** — 分块上传，进度追踪
- **隐私安全** — 可选的访问密码保护

## 技术栈

- **框架**: Tauri v2 (Rust + WebView2)
- **前端**: Vanilla JS (ES Modules) + CSS 自定义属性
- **手机端页面**: 纯 HTML/JS/CSS，由内置 HTTP 服务器提供
- **HTTP 服务**: tiny_http (Rust)

## 项目结构

```
Su/
├── src/                    # 桌面前端
│   ├── index.html          # 主窗口
│   ├── js/                 # JavaScript 模块
│   │   ├── main.js         # 入口
│   │   ├── i18n.js         # 多语言
│   │   ├── state.js        # DOM 引用与应用状态
│   │   ├── utils.js        # Toast、剪贴板、二维码工具
│   │   ├── settings.js     # 设置页面
│   │   ├── share.js        # 文件分享 UI
│   │   ├── received.js     # 接收记录 UI
│   │   └── theme.js        # 主题与外观
│   ├── css/
│   │   └── styles.css      # 全部样式
│   └── assets/
│       └── fonts/          # Font Awesome (本地)
├── src-tauri/              # Rust 后端
│   ├── src/                # Rust 源码
│   │   ├── main.rs         # 入口
│   │   ├── lib.rs          # 应用初始化与 Tauri 命令注册
│   │   ├── http.rs         # HTTP 服务器与路由
│   │   ├── commands.rs     # Tauri 命令
│   │   ├── state.rs        # 应用状态与持久化
│   │   ├── sound.rs        # 音效播放
│   │   ├── qr.rs           # 二维码生成
│   │   ├── com_shellext.rs # Windows 右键菜单扩展
│   │   └── utils.rs        # 工具函数
│   ├── web/                # 手机端页面
│   │   ├── send.html       # 发送页面
│   │   ├── download.html   # 单文件下载
│   │   └── bundle_multi.html # 多文件下载
│   ├── sounds/             # 提示音文件
│   └── icons/              # 应用图标
├── package.json
└── README.md
```

## 构建

### 环境要求

| 工具 | 说明 |
|------|------|
| [Node.js](https://nodejs.org/) >= 18 | JavaScript 运行时 |
| [Rust](https://www.rust-lang.org/tools/install) >= 1.70 | `rustup` 安装，包含 `cargo` |
| WebView2 | Windows 10/11 已内置，Win 7 需手动安装 |

### 开发

```bash
npm install
npm run tauri dev
```

### 构建安装包

```bash
npm run tauri build
```

构建产物：

| 路径 | 说明 |
|------|------|
| `src-tauri/target/release/su.exe` | 绿色版（免安装） |
| `src-tauri/target/release/bundle/msi/` | MSI 安装包 |
| `src-tauri/target/release/bundle/nsis/` | NSIS 安装包 |

## 使用

1. 启动 Su!
2. 电脑端：拖拽文件到窗口生成二维码，手机扫码下载
3. 手机端：扫码进入发送页面，选择文件发送到电脑
4. 接收的文件保存在 `下载/Su/` 文件夹
5. 右键菜单：在设置中注册后，右键任意文件 → 通过 Su! 分享

## 许可证

MIT License — 详见 [LICENSE](LICENSE)

## 贡献者

[![Contributors](https://contrib.rocks/image?repo=Hunter-Lies/Su)](https://github.com/Hunter-Lies/Su/graphs/contributors)

## 作者

**HunterLies** — [Bilibili](https://space.bilibili.com/488494586) · [官网](https://htovo.com)
