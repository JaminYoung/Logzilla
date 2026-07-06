<div align="center">

<img src="src-tauri/icons/icon.svg" width="120" alt="Logzilla logo" />

# Logzilla

**蓝牙设备串口日志打印与在线分析工具**

一个集串口日志打印、关键词过滤、匹配高亮，蓝牙协议解析、音频提取与解码于一体的桌面工具箱

[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18.3-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
![Platform](https://img.shields.io/badge/Platform-Windows%20(x86)-0078D6?logo=windows&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

</div>

---

## 📖 简介

**Logzilla** 是一款专门面向蓝牙设备开发/调试的桌面工具，基于 **Tauri 2 + React + Rust** 构建。它把日常调试中零散的工具整合到一个界面里：实时串口日志、模糊/正则搜索、关键词高亮与过滤、HCI 协议解析、LC3 音频编解码、USB 音频提取、~~固件配置文件解析与编辑~~等。

> 📐 想了解内部实现细节，请阅读 [**ARCHITECTURE.md**](./ARCHITECTURE.md)（架构深潜文档）。

## ✨ 功能特性

### 🖥️ 串口日志
- **多串口连接**，支持单双窗口切换，最多支持双窗口同时输出日志，天生适用于TWS耳机调试场景

### 🔍 搜索 · 高亮 · 过滤
- **三种搜索模式**：模糊搜索（Rust → WASM `nucleo` 引擎）/ 正则 / 普通匹配，支持自定义搜索范围
- **关键词高亮**：自定义高亮规则
- **日志过滤**：按规则过滤，并可从过滤视图**一键定位**回原始日志行

### 📡 HCI 协议
- **协议解析**：支持基础的LMP / LLCP / HCI 命令 / HCI 事件 / LE 子事件解析，快速读懂日志，省去导出HCI日志到解析工具的麻烦（右键“这是什么”打开该功能）
- **HCI 提取**：从日志中提取 HCI 帧，支持根据电脑时间修正HCI日志时间戳
- **HCI 动态加载（Live Import）**：把日志中的 HCI 帧**实时注入** Frontline/Wireless Protocol Suit分析器

### 🎵 音频工具
- **LC3 工具箱**：支持串口实时捕获 LC3 码流/本地导入LC3编码RAW文件 → 解码 → 导出 **WAV / RAW**
- **USB 音频提取**：支持从wireshark抓取的日志，自动解析UAC音频描述符并提取音频流

### ⚙️ 配置编辑
- ~~**DCF 配置文件解析与编辑**：48 字节头 + XCFG 配置树（`SUB`/`CHK`/`LVL`/`LST`/`U08`/`U16`/`UBT`/`TXT`/`MAC` 等 11 种标签）+ INFO 区读写 + 校验和~~
- ~~书签导航、修改即时应用、落盘保存~~

### 🔥 固件烧录
- （*尚未实现，欢迎各位接力开发*）

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | React 18.3 + TypeScript 5.5 |
| UI / 样式 | Tailwind CSS 4.3 · Motion (Framer Motion) · Lucide 图标 |
| 状态管理 | React Hooks（useState / useRef） |
| 模糊搜索 | `nucleo-matcher` → WASM |
| 后端框架 | Tauri 2 |
| 后端语言 | Rust 2021（`serialport` / `tokio` / `libloading` / `regex` …） |
| 构建工具 | Vite 6（多页面构建） |
| 目标平台 | Windows 32 位（`i686-pc-windows-msvc`） |

## 📂 目录结构

```
Logzilla/
├── src/                     # React 前端
│   ├── App.tsx              # 主应用：全局状态 + 日志轮询
│   ├── main.tsx / lc3.tsx   # 主页 / LC3 工具箱页 入口（多页面）
│   ├── components/          # UI 组件（标题栏 / 工具栏 / 日志面板 / 搜索 / 配置 …）
│   │   ├── LC3ToolKit/      # LC3 音频工具箱
│   │   └── USBAudioExtractor/
│   ├── hooks/               # useWindowMoving 等
│   ├── utils/               # 前端搜索调度
│   └── wasm/                # nucleo-wasm 模糊搜索（编译产物）
│
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── lib.rs           # AppState 定义 + 命令注册
│   │   ├── commands/        # Tauri 命令层（前端调用入口）
│   │   └── core/            # 核心业务：dcf / xcfg / config / serial /
│   │                        #          hci / lc3 / usb_audio
│   ├── nucleo-wasm/         # 模糊搜索子 crate（Rust → WASM）
│   ├── lc3.dll              # LC3 解码运行时依赖（32 位厂商 DLL）
│   └── tauri.conf.json
│
├── index.html / lc3.html    # 多页面 HTML 入口
├── vite.config.ts
└── ARCHITECTURE.md          # 架构深潜文档
```

## 🚀 构建与运行

### 环境要求
- **Node.js** ≥ 18
- **Rust** stable，并添加 32 位 target：`rustup target add i686-pc-windows-msvc`
- **Visual Studio Build Tools 2022**（C++ 桌面开发工作负载）

### 开发模式（带热更新）
```bash
npm install
npm run tauri dev
```

### 生产构建（32 位）
```bash
npm run tauri build -- --target i686-pc-windows-msvc
# 产物：src-tauri/target/i686-pc-windows-msvc/release/logzilla.exe
```
### 一键编译
使用build.cmd 一键安装依赖和自动编译

> 💡 HCI LIVE功能目前只调通32位的版本，后端**必须** 32 位构建，不能使用默认 64 位 target。

## ⚠️ 已知限制

1. **烧录功能**：UI 与进度条为占位，未实现真实串口烧录协议。
2. **32 位约束**：HCI LIVE功能依赖的动态库LiveImport.dll只有32位调通，64位版本暂未解决，LC3解码库目前也是以32位来构建，整个后端必须以 32 位构建。

## 📚 相关文档

- [ARCHITECTURE.md](./ARCHITECTURE.md) — 完整架构、数据流、关键机制、DCF 文件格式说明

## 📝 许可证

本项目基于 [MIT License](./LICENSE) 开源。

---

<div align="center">
<sub>Built with 🦀 Rust + ⚛️ React + 🖼️ Tauri</sub>
</div>
