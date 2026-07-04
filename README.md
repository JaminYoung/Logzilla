<div align="center">

<img src="src-tauri/icons/icon.svg" width="120" alt="Logzilla logo" />

# Logzilla

**蓝牙芯片串口日志打印与在线分析工具**

一个集串口日志、协议解析、音频提取、配置编辑于一体的桌面工具箱

[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18.3-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
![Platform](https://img.shields.io/badge/Platform-Windows%20(x86)-0078D6?logo=windows&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

</div>

---

## 📖 简介

**Logzilla** 是一款面向蓝牙芯片开发/调试的桌面工具，基于 **Tauri 2 + React + Rust** 构建。它把日常调试中零散的工具整合到一个界面里：实时串口日志、模糊/正则搜索、关键词高亮与过滤、HCI 协议解析、LC3 音频编解码、USB 音频提取、DCF 配置文件解析与编辑等。

前端采用自定义标题栏 + 亚克力毛玻璃 UI，后端为 Rust，通过 Tauri IPC 桥接。由于需要加载 32 位厂商 DLL（如 `lc3.dll`），整个应用以 **32 位 Windows（`i686-pc-windows-msvc`）** 为构建目标。

> 📐 想了解内部实现细节，请阅读 [**ARCHITECTURE.md**](./ARCHITECTURE.md)（架构深潜文档）。

## ✨ 功能特性

### 🖥️ 串口日志
- **多串口连接**，双面板并排对比不同端口日志
- **万行级流畅渲染**：`DomLogList` 直接 DOM 渲染 + `requestAnimationFrame` 批量追加，绕过 React diff 开销
- 可选**时间戳**（`HH:MM:SS.fff`），自动跟随滚动到底部，上滑时暂停跟随
- **自动保存**日志到文件，缓冲区超限自动裁剪且不丢显示（绝对行号游标机制）
- **窗口拖动性能优化**：拖动/缩放期间自动关闭毛玻璃模糊，零 React re-render

### 🔍 搜索 · 高亮 · 过滤
- **三种搜索模式**：模糊搜索（Rust → WASM `nucleo` 引擎）/ 正则 / 普通匹配，支持多视图作用域
- **关键词高亮**：自定义高亮规则
- **日志过滤**：按规则过滤，并可从过滤视图**一键定位**回原始日志行

### 📡 HCI 协议
- **协议解析**：LMP / LLCP / HCI 命令 / HCI 事件 / LE 子事件，字段级悬浮提示（`ProtocolTooltip`）
- **HCI 提取**：从日志中提取 HCI 帧（时间戳 carry-forward、GBK 容错读取），一键在资源管理器定位
- **HCI 动态加载（Live Import）**：把日志中的 HCI 帧**实时注入** Frontline WPS/FTS 分析器；自动启动 WPS、轮询就绪状态，WPS 关闭时自动停用

### 🎵 音频工具
- **LC3 工具箱**（独立窗口）：串口捕获 LC3 码流 / 导入文件 → 解码 → 导出 **WAV / RAW**，基于 32 位 `lc3.dll` FFI
- **USB 音频提取**：从 `usbpcap` 抓包解析 USB 音频描述符并提取音频流

### ⚙️ 配置编辑
- **DCF 配置文件解析与编辑**：48 字节头 + XCFG 配置树（`SUB`/`CHK`/`LVL`/`LST`/`U08`/`U16`/`UBT`/`TXT`/`MAC` 等 11 种标签）+ INFO 区读写 + 校验和
- **条件可见性**：配置项可绑定 `ui_condition_var`，联动显示/隐藏
- 书签导航、修改即时应用、落盘保存

### 🔥 固件烧录
- 烧录 UI 与进度条已就绪（*真实串口烧录协议为占位，尚未实现*）

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

> 💡 由于依赖 32 位 `lc3.dll` 等厂商组件，后端**必须** 32 位构建，不能使用默认 64 位 target。

## ⚠️ 已知限制

1. **烧录功能**：UI 与进度条为占位，未实现真实串口烧录协议。
2. **32 位约束**：`lc3.dll` 等为 32 位，整个后端必须以 32 位构建。
3. **MSI 打包**：图标资源不完整，目前仅能产出独立 `exe`。

## 📚 相关文档

- [ARCHITECTURE.md](./ARCHITECTURE.md) — 完整架构、数据流、关键机制、DCF 文件格式说明

## 📝 许可证

本项目基于 [MIT License](./LICENSE) 开源。

> ⚠️ 注意：仓库内含的第三方 32 位组件（`lc3.dll` 等）不在 MIT 授权范围内，其分发与使用请遵循各自厂商的授权条款。

---

<div align="center">
<sub>Built with 🦀 Rust + ⚛️ React + 🖼️ Tauri</sub>
</div>
