# Logzilla 项目架构文档

## 1. 项目概述

Logzilla 是一个基于 **Tauri 2 + React + Rust** 的蓝牙芯片串口日志打印与在线分析工具。除串口日志外，还集成了 LC3 音频编解码工具箱、USB 音频提取、HCI 动态加载（注入 WPS/FTS）、模糊/正则搜索、关键词高亮、日志过滤、DCF 配置文件解析与编辑、固件烧录（占位）等功能。

- **构建目标**：32 位 Windows（`i686-pc-windows-msvc`），因需加载 32 位厂商 DLL。
- **窗口**：自定义标题栏（`decorations: false`），多页面 Vite 构建（主页 + LC3 工具箱页）。

## 2. 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 前端框架 | React + TypeScript | 18.3 / 5.5 |
| UI 样式 | Tailwind CSS | 4.3 |
| 动画 | Motion (Framer Motion) | 12.x |
| 图标 | Lucide React | 1.x |
| 状态管理 | React useState/useRef（Zustand 预留未用） | 5.x |
| 模糊搜索 | nucleo-wasm（Rust→WASM） | vendored |
| 后端框架 | Tauri | 2.x |
| 后端语言 | Rust | 1.95+ |
| 串口通信 | serialport crate | 4.x |
| 构建 | Vite | 6.3 |
| 平台 | Windows（32 位） | - |

## 3. 目录结构

```
Logzilla/
├── src/                          # React 前端源码
│   ├── App.tsx                   # 主应用，全局状态 + 轮询拉取日志
│   ├── main.tsx                  # 主页 React 入口
│   ├── lc3.tsx                   # LC3 工具箱页入口（多页面构建）
│   ├── components/               # UI 组件
│   │   ├── TitleBar.tsx          # 自定义标题栏（拖拽/最小化/最大化/关闭）
│   │   ├── SecondaryToolbar.tsx  # 工具栏（串口/波特率/烧录/双窗/主题）
│   │   ├── LeftSideMenu.tsx      # 左侧滑出菜单（时间戳/过滤/高亮/字体/动态加载等）
│   │   ├── LogPanel.tsx          # 日志面板（DomLogList 直接 DOM 渲染 + rAF 滚动跟随）
│   │   ├── SearchBar.tsx         # 搜索栏（模糊/正则/普通，多视图作用域）
│   │   ├── FilterSettingsDialog.tsx   # 过滤规则设置
│   │   ├── HighlightSettingsDialog.tsx# 高亮规则设置
│   │   ├── ConfigPanel.tsx       # 配置编辑面板（书签导航/条件可见性）
│   │   ├── ConfigToggle.tsx / ConfigToggleClose.tsx  # 配置面板开关按钮
│   │   ├── SerialDropdown.tsx    # 串口下拉选择（带独立开关）
│   │   ├── ProgressBar.tsx       # 烧录进度条
│   │   ├── StatusBar.tsx         # 底部状态栏
│   │   ├── Toast.tsx             # 通知提示
│   │   ├── ProtocolTooltip.tsx   # HCI 协议字段悬浮提示
│   │   ├── Button.tsx            # 通用按钮
│   │   ├── LC3ToolKit/           # LC3 工具箱（独立窗口/捕获/解码/查看）
│   │   └── USBAudioExtractor/    # USB 音频提取对话框
│   ├── hooks/
│   │   └── useWindowMoving.ts    # 窗口拖动/缩放检测（CSS data 属性方式，零 re-render）
│   ├── utils/
│   │   └── search.ts             # 前端搜索调度（调用 WASM）
│   ├── styles/                   # index.css + theme.css（主题变量 + 拖动期禁用模糊）
│   ├── types/                    # TypeScript 类型（预留）
│   └── wasm/                     # nucleo-wasm 模糊搜索（vendored 编译产物）
│
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml / Cargo.lock   # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置（decorations:false, bundle lc3.dll）
│   ├── build.rs                  # 构建脚本
│   ├── lc3.dll                   # LC3 解码 FFI 运行时依赖（厂商 32 位 DLL）
│   ├── bin/
│   │   └── live_import_helper.exe# HCI 动态加载辅助程序（运行时调用）
│   ├── capabilities/             # Tauri 权限配置
│   ├── icons/                    # 应用图标
│   ├── gen/                      # Tauri 生成的 schema
│   ├── src/
│   │   ├── main.rs               # Rust 入口
│   │   ├── lib.rs                # AppState 定义 + 命令注册
│   │   ├── commands/             # Tauri 命令层（前端调用入口）
│   │   │   ├── dcf_commands.rs       # DCF 打开/保存
│   │   │   ├── serial_commands.rs    # 串口打开/关闭/读写（日志缓冲 + 行数裁剪游标）
│   │   │   ├── config_commands.rs    # 配置值读写
│   │   │   ├── flash_commands.rs     # 烧录控制（占位）
│   │   │   ├── file_commands.rs      # 文件读写
│   │   │   ├── hci_commands.rs       # HCI 日志提取 / 资源管理器定位
│   │   │   ├── hci_parser_commands.rs# 协议行解析（LMP/LLCP/HCI 命令/事件）
│   │   │   ├── lc3_commands.rs       # LC3 捕获/解码/导出
│   │   │   ├── live_import_commands.rs# HCI 动态加载（WPS/FTS 注入）
│   │   │   ├── search_commands.rs    # 日志拉取(get_logs)/更新/清空 + 搜索
│   │   │   └── usb_audio_commands.rs # USB 音频 pcap 分析/提取
│   │   └── core/                 # 核心业务逻辑
│   │       ├── dcf/              # DCF 文件格式（48 字节头 + INFO 定位）
│   │       ├── xcfg/             # XCFG 配置树解析（11 种标签）
│   │       ├── config/           # INFO 数据读写 + 校验和
│   │       ├── serial/           # 串口句柄封装
│   │       ├── flash/            # 烧录引擎/协议/进度
│   │       ├── hci/              # HCI 协议解析（命令/事件/LE 子事件/LLCP/LMP 表）
│   │       ├── lc3/              # LC3 解码器 FFI / 导入 / 导出
│   │       ├── search/           # 搜索引擎 + 类型
│   │       └── usb_audio/        # USB 音频 pcap 读取 / 描述符 / 提取
│   ├── nucleo-wasm/              # 模糊搜索子 crate（Rust→WASM）
│   └── live_import_helper/       # HCI 动态加载辅助程序子 crate
│
├── index.html / lc3.html         # 多页面 HTML 入口
├── package.json / package-lock.json
├── vite.config.ts                # Vite 多页面构建配置
├── tsconfig.json / postcss.config.js
└── ARCHITECTURE.md
```

## 4. 架构设计

### 4.1 整体架构

```
┌─────────────────────────────────────────────────┐
│                   React 前端                      │
│  App.tsx (状态中心 + 日志轮询) ──→ 各 UI 组件     │
│       │                                          │
│       │ invoke() / listen()                      │
│       ▼                                          │
├─────────────────────────────────────────────────┤
│               Tauri IPC 桥接层                    │
│  #[tauri::command] 函数                           │
├─────────────────────────────────────────────────┤
│                   Rust 后端                       │
│  commands/ ──→ core/ (业务逻辑)                   │
│       │                                          │
│       ▼                                          │
│  AppState (Mutex 保护的全局状态)                   │
│  + 子 crate: nucleo-wasm / live_import_helper    │
│  + FFI: lc3.dll (32 位)                          │
└─────────────────────────────────────────────────┘
```

### 4.2 数据流

**串口日志流程：**
```
后端 read_serial_data（定时轮询串口）
  → 按行拆分（跨次读取用 partial_lines 缓冲拼接）
  → 可选加时间戳 (HH:MM:SS.fff)
  → 写入 logs/logs2 缓冲（超过 max_lines 从头 drain，累加 base 游标）
  → 版本号 +1
前端 fetchLogs（useEffect 稳定轮询，不依赖 version）
  → invoke("get_logs", { panel, sinceVersion, knownLines })
  → 后端用 known_lines - base 映射缓冲下标，返回新增行
  → setLogs(prev => [...prev, ...newLogs])，累计 logsLenRef
```

**DCF 配置流程：**
```
用户选文件 → invoke("open_dcf")
  → dcf::parser 解析头部 → xcfg::parser 解析配置树 + INFO
  → xcfg::offset 计算偏移 → 返回 DcfInfo
用户改值 → 记录 changes → invoke("apply_config_changes")
  → config::editor 写 INFO → invoke("save_dcf") 落盘
```

**HCI 动态加载流程：**
```
用户开启动态加载 → invoke("live_import_init", { wpsPath })
  → 加载 FTS DLL → 必要时 launch_wps → 轮询 IsAppReady
  → 就绪后定时把日志中的 HCI 帧解析出来
  → invoke("live_import_send_frame") 注入 WPS
  → 监控 Fts.exe，WPS 关闭则自动停用
```

### 4.3 全局状态 (AppState)

`lib.rs` 中定义，`Mutex` 保证线程安全：

| 字段 | 类型 | 说明 |
|------|------|------|
| `dcf_data` / `dcf_path` | `Option<...>` | DCF 原始字节 / 路径 |
| `config_tree` | `Vec<ConfigItem>` | 解析后的配置树 |
| `info_data` / `info_offset` / `info_len` | `Vec<u8>` / `usize` | INFO 区域数据与定位 |
| `flash_engine` | `FlashEngine` | 烧录引擎 |
| `serial_ports` | `HashMap<String, Arc<SerialPortHandle>>` | 已打开串口 |
| `connected_ports` | `Vec<String>` | 已连接端口名 |
| `logs` / `logs2` | `Vec<String>` | 面板 1/2 日志缓冲 |
| `logs_version` / `logs2_version` | `u64` | 日志版本号（变更递增） |
| `logs_base` / `logs2_base` | `usize` | 已丢弃行数（绝对游标，见 §5.2） |
| `partial_lines` | `HashMap<String, String>` | 跨次读取的不完整行缓冲 |
| `lc3_capture_buffers` / `lc3_capture_active` | `HashMap` / `Vec` | LC3 捕获缓冲与活动端口 |

### 4.4 前端状态 (App.tsx)

React `useState` 管理 UI 状态；**日志版本/行数游标用 `useRef`**（非 state），避免轮询 effect 随 version 变化销毁重建。

| 状态 / ref | 说明 |
|------|------|
| `logs` / `logs2` | 面板 1/2 日志（state，触发渲染） |
| `logsVersionRef` / `logs2VersionRef` | 版本号游标（ref，不触发渲染） |
| `logsLenRef` / `logs2LenRef` | 前端累计行数（ref） |
| `logDisplayPort` / `logDisplayPort2` | 面板 1/2 绑定串口 |
| `dualPanelMode` | 双面板开关 |
| `searchQuery` / `searchMode` / `searchScope` | 搜索状态 |
| `filterRules` / `highlightRules` | 过滤/高亮规则 |
| `liveImport*` | HCI 动态加载状态 |

## 5. 关键机制

### 5.1 窗口拖动性能优化

拖动/缩放窗口时，毛玻璃（`backdrop-filter: blur`）会触发昂贵 GPU 合成，大量日志时还会因 `scrollTop = scrollHeight` 强制布局而卡顿。

- `useWindowMoving` 同时监听 `onMoved` + `onResized`，拖动期在 `document.documentElement` 上设 `data-window-moving="true"`（**纯 DOM 属性，零 React re-render**）。
- CSS：`[data-window-moving="true"] .acrylic-bar { backdrop-filter: none; background: var(--acrylic-solid); }` 拖动期禁用模糊。需要条件模糊的元素加 `acrylic-bar` class。
- `DomLogList` 的 rAF 循环读取该属性，拖动期跳过 DOM append 与 `scrollTop` 赋值。

### 5.2 日志缓冲行号游标（防裁剪后不显示）

缓冲超过 `max_lines` 从头 `drain` 后，前端累计行号与缓冲下标会错位，导致 `get_logs` 返回空、日志停住。

- 后端维护绝对游标 `logs_base`：`drain` 时 `base += drain_count`，`clear_logs` 时 `base = 0`。
- `get_logs` 用 `known_lines - base`（`saturating_sub`）映射缓冲内起始下标。
- 前端 `logsLenRef` 只增不减，清空时由 `handleClearLogs` 同步重置 ref + 后端缓冲。

### 5.3 DomLogList 直接 DOM 渲染

日志行不走 React diff，而由 `requestAnimationFrame` 永续循环把 `pendingRef` 里的新行批量 `appendChild` 到容器，避免万行级 React 重渲染开销。用户上滚时 `isAtBottomRef=false` 停止自动跟底，滚回底部后恢复跟随。

### 5.4 多页面与独立窗口

- Vite 多页面构建：`index.html`（主页）+ `lc3.html`（LC3 工具箱）。
- LC3 工具箱通过 `WebviewWindow` 开独立窗口（`/lc3.html`，无装饰、固定宽）。
- 主窗口与 LC3 窗口通过 `lc3-port-changed` 事件同步串口占用。

## 6. DCF 文件格式

```
┌──────────────────┐ 0x00
│   DCF Header     │ 48 字节 (version, info_offset, info_len)
├──────────────────┤ 0x30  Padding
├──────────────────┤ 0x50  XCFG Config Tree (SUB/CHK/LVL/LST...)
├──────────────────┤ info_offset   "INFO" Marker (4 字节)
├──────────────────┤ info_offset+16  INFO Data (配置值存储)
└──────────────────┘
```

### 6.1 配置树标签类型

| 标签 | 说明 | 值类型 |
|------|------|--------|
| `SUB` | 子菜单分组 | - |
| `CHK` | 复选框 | `bool` (1 字节) |
| `LVL` | 条件级别 | `u32` |
| `LST` / `LSV` | 下拉列表 | `u32` |
| `U08`/`S08`/`U16` | 整数 | `u8`/`i8`/`u16` |
| `UBT` | 位域 | 位操作 |
| `TXT` | 文本 | `string` |
| `MAC` | MAC 地址 | `[u8; 6]` |

### 6.2 条件可见性

配置项可带 `ui_condition_var`，当所指变量为真时该项可见（例：`func_aux_en` 控制 8 个 AUX 项）。

## 7. 构建与运行

### 7.1 开发

```bash
npm install
npm run tauri dev          # 带热更新
```

### 7.2 生产构建（32 位）

```bash
npm run tauri build -- --target i686-pc-windows-msvc
# exe: src-tauri/target/i686-pc-windows-msvc/release/logzilla.exe
```

> MSI 打包需 `.ico` 图标，当前缺失，仅 exe 构建成功。

### 7.3 依赖要求

- Node.js >= 18
- Rust stable + `i686-pc-windows-msvc` target
- Visual Studio Build Tools 2022（C++ 桌面工作负载）

## 8. 已知限制

1. **烧录功能**：UI 占位，未实现真实串口烧录协议。
2. **32 位约束**：`lc3.dll` 等为 32 位，整个后端必须 32 位构建。
3. **MSI 图标**：缺 `.ico`，仅能产出 exe。

## 9. 关键文件速查

| 功能 | 文件路径 |
|------|----------|
| 应用入口/状态 | `src/App.tsx` |
| 后端状态/命令注册 | `src-tauri/src/lib.rs` |
| 日志缓冲/裁剪游标 | `src-tauri/src/commands/serial_commands.rs` |
| 日志拉取/清空 | `src-tauri/src/commands/search_commands.rs` |
| 日志 DOM 渲染/滚动 | `src/components/LogPanel.tsx` (DomLogList) |
| 窗口拖动优化 | `src/hooks/useWindowMoving.ts` |
| DCF 解析 | `src-tauri/src/core/dcf/parser.rs` |
| 配置树解析 | `src-tauri/src/core/xcfg/parser.rs` |
| HCI 协议解析 | `src-tauri/src/core/hci/parser.rs` |
| LC3 解码 FFI | `src-tauri/src/core/lc3/ffi.rs` |
| HCI 动态加载 | `src-tauri/src/commands/live_import_commands.rs` |
| USB 音频提取 | `src-tauri/src/core/usb_audio/extractor.rs` |
