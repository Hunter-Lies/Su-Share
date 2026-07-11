<p align="center"><img src="logo.png" width="128" alt="Su!"></p>

<h1 align="center">Su! — LAN File Transfer</h1>

<p align="center">
  <a href="README.md"><img src="https://img.shields.io/badge/中文-README-red?style=for-the-badge" alt="中文"></a>
</p>

<p align="center"><em>咻一下，就到了！ Whoosh, it's there!</em></p>

<p align="center">
  <img src="screenshot.png" width="600" alt="Screenshot">
</p>

<p align="center">
  <a href="https://github.com/Hunter-Lies/Su/releases/latest"><img src="https://img.shields.io/github/v/release/Hunter-Lies/Su?style=flat-square" alt="Release"></a>
  <a href="https://github.com/Hunter-Lies/Su/releases/latest"><img src="https://img.shields.io/github/downloads/Hunter-Lies/Su/total?style=flat-square" alt="Downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/Hunter-Lies/Su?style=flat-square" alt="MIT"></a>
  <a href="https://github.com/Hunter-Lies/Su/stargazers"><img src="https://img.shields.io/github/stars/Hunter-Lies/Su?style=flat-square" alt="Stars"></a>
</p>

---

**Su!** is a lightweight LAN file transfer tool. No cloud, no login, no app install — scan a QR code and files fly between your phone and PC. Or right-click any file, generate a QR code, and your phone downloads it instantly.

## Download

| Version | Link | For |
|---------|------|-----|
| 64-bit | [Su-v1.2.0-windows-x64.zip](https://github.com/Hunter-Lies/Su/releases/latest/download/Su-v1.2.0-windows-x64.zip) | Modern Windows 10/11 |
| 32-bit | [Su-v1.2.0-windows-x86.zip](https://github.com/Hunter-Lies/Su/releases/latest/download/Su-v1.2.0-windows-x86.zip) | Legacy / 32-bit systems |

> Unzip and run `su.exe` — **no installation required**. If SmartScreen blocks it, click "More info" -> "Run anyway".
>
> macOS / Linux users can build from source (see Build section below). Some features (context menu, acrylic glass) are Windows-only.

## Features

- **QR Share** — Right-click any file, generate a QR code, phone scans to download
- **Phone -> PC** — Send files from your phone browser over LAN
- **Batch Download** — Select all, single, or batch with clear file list
- **Resumable Transfer** — Range-based chunked download; close the browser and pick up where you left off
- **Smart Batching** — Auto device detection (iPhone / Android / Windows / Mac / Linux), grouped by batch
- **5 Languages** — 简体中文 · 繁體中文 · English · 日本語 · 한국어, mobile pages auto-detect
- **Dark Mode** — Classic & frosted glass themes, follows system
- **Sound + Popup** — Audio alerts and card notifications on receive
- **System Tray** — Minimize to tray, auto-start with system
- **Portable** — Single exe, unzip and run, no registry writes

## Quick Start

1. [Download](https://github.com/Hunter-Lies/Su/releases/latest) the zip for your system, unzip
2. Double-click `su.exe` — the window opens with a QR code
3. **Phone -> PC**: Phone scans QR -> select files to send
4. **PC -> Phone**: Drag files to the window -> phone scans QR to download

## Tech Stack

| Layer | Stack |
|-------|-------|
| Desktop Framework | [Tauri v2](https://tauri.app/) + Rust |
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
npm run tauri dev                         # Development
npm run tauri build                       # Release (x64)
npm run tauri build -- --target i686-pc-windows-msvc  # Release (x86)
```

Output in `src-tauri/target/release/`.

## Project Structure

```
Su/
├── src/                          # Desktop Frontend
│   ├── index.html                # Main window
│   ├── css/
│   │   └── styles.css            # All styles
│   ├── js/                       # JavaScript modules
│   │   ├── main.js               # Entry point
│   │   ├── i18n.js               # i18n (zh-CN/zh-TW/en/ja/ko)
│   │   ├── state.js              # DOM refs & app state
│   │   ├── utils.js              # Toast, clipboard, QR helpers
│   │   ├── settings.js           # Settings page
│   │   ├── share.js              # File sharing UI
│   │   ├── received.js           # Received records UI
│   │   └── theme.js              # Theme & appearance
│   └── assets/
│       └── fonts/                # Font Awesome 6 (local)
├── src-tauri/                    # Rust Backend
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri config
│   ├── build.rs                  # Build script
│   ├── capabilities/
│   │   └── default.json          # Tauri permissions
│   ├── icons/                    # App icons
│   ├── sounds/                   # Audio files (mp3)
│   ├── web/                      # Mobile Pages
│   │   ├── send.html             # Upload page
│   │   ├── download.html         # Single file download
│   │   ├── bundle_multi.html     # Multi-file download
│   │   ├── i18n.js               # Mobile i18n
│   │   └── fonts/                # Font Awesome (mobile)
│   ├── src/                      # Rust Source
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Setup, commands, tray
│   │   ├── http.rs               # HTTP server, routes, upload/download
│   │   ├── commands.rs           # Tauri commands (file ops, settings, context menu)
│   │   ├── state.rs              # App state & persistence (JSON)
│   │   ├── sound.rs              # Audio playback
│   │   ├── qr.rs                 # QR code generation
│   │   ├── com_shellext.rs       # Windows context menu extension
│   │   └── utils.rs              # Utils (MIME, formatting, response)
│   ├── AppxManifest.xml          # Windows context menu manifest
│   ├── register_sparse.ps1       # Context menu register script
│   └── unregister_sparse.ps1     # Context menu unregister script
├── package.json                  # Node dependencies
├── logo.png                      # Logo
├── screenshot.png                # Screenshot
├── README.md                     # 中文 (default)
├── README_en.md                  # English
└── LICENSE                       # MIT License
```

## Contributors

[![Contributors](https://contrib.rocks/image?repo=Hunter-Lies/Su)](https://github.com/Hunter-Lies/Su/graphs/contributors)

Issues and PRs welcome!

## License

MIT — see [LICENSE](LICENSE)

## Author

**HunterLies** · [Bilibili](https://space.bilibili.com/488494586) · [Website](https://htovo.com)
