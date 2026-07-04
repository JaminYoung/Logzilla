# Logzilla HCI Live Import — 问题总结（第二次更新）

## 问题描述
- **现象**：`SendFrame` 全部返回 `hr=0`，但 WPS Virtual Sniffer Summary 面板始终为空
- **环境**：Windows，WPS 安装在 `C:\software\WPS`
- **核心困惑**：所有 API 调用都返回成功（hr=0x0），但没有任何数据到达 WPS

## 当前状态（2026-05-23 更新）

### 最新发现：官方示例开发套件

`C:\software\WPS\Live Import Developers Kit\` 包含了完整的官方示例源码：

| 组件 | 文件 |
|------|------|
| API 头文件 | `h\LiveImportAPI.h`（832 行，完整函数定义） |
| 通知类型 | `h\drivernotifications.h`（全部通知枚举） |
| 队列模式 | `h\QueueMode.h` |
| Straight C 示例 | `Straight C Sample\csample.c`（不启动 FTS，需手动启动） |
| GUI 示例 | `GUI Sample\guisample.cpp` + `maindialog.cpp`（MFC 对话框） |
| C++ 包装器 | `Wrapper Sample\liveimportcwrapper.cpp` / `.h` |
| 示例 INI | `GUI Sample\guisample.ini`（含动作数据格式） |
| 预编译 EXE | `Straight C Sample\release\csample.exe` |
| 用户手册 | `UserManualLiveImport.pdf`（完整 PDF） |

### 关键发现

#### 1. `SetExePath` 在官方示例中从未被调用
- `csample.c`、`guisample.cpp`、`maindialog.cpp` 均不调用 `SetExePath`
- 示例直接调用 `InitializeLiveImport` → `IsAppReady` → `SendFrame`
- Android AOSP `btsnoop_live.py` 也不调用 `SetExePath`
- **SetExePath 似乎是可选的**，不调用也不影响正常工作流

#### 2. `IsAppReady` 需要 FTS 已经开始捕获
- csample.exe 的错误信息："Make sure you have started fts.exe for the Virtual Sniffing capture option and have selected **'Start Capture'**"
- `SendNotification(eStartCaptureToFile)` 可以程序化地通知 FTS 开始捕获
- 但我们的测试显示：即使调用了 `SendNotification(eStartCaptureToFile)` → hr=0x0，`IsAppReady` 在 60 秒后**仍返回 FALSE**

#### 3. `IsAppReady` 参数类型是 `bool*` 不是 `int*`
```c
typedef HRESULT(IsAppReady)(bool* pboolIsAppReady);
```
C++ `bool` 是 1 字节。当前 Rust 使用 `&mut i32`（4 字节），但理论上不影响（只检查 !=0）。

#### 4. 数据格式：`[Side, drf] + HCI_payload`
从 `guisample.ini` 的 Action 数据确认的数据格式：
```
Action1Data=0x00,0x01,0x09,0x10,0x00
           ^^^^ ^^^^ ^^^^^^^^^^^^^^^^
           Side  drf  HCI_Read_BD_ADDR (no H4 byte)
```
- 字节 0（Side）：`0x00=Host(sent)`，`0x01=Controller(received)`
- 字节 1（drf）：`1=Command`，`2=ACL`，`8=Event`
- 字节 2+：纯 HCI 负载（**不含 H4 类型字节**）

`SendFrame` 的 `iSide` 参数名（来自头文件），不是 `iStream`：
```c
typedef HRESULT(SendFrame)(
    int iOriginalLength,
    int iIncludedLength,
    const BYTE* pbytFrame,
    int iDrf,
    int iSide,              // 原名 iStream
    int64_t i64Timestamp100ns);  // FILETIME: 100ns since 1601-01-01
```

#### 5. 连接字符串（ConnectionString）可能是最关键的问题
- **官方示例**使用：`FTS4BT Live Import.`（简单名称）
- **WPS 安装**使用：`Wireless Protocol Suite Live Import.FDFFFFFF!C2B6C644597ABC4C0DD86DF3C87E5A142DD292;0713D32E`（含 GUID）
- **假设**：FTS 在 Virtual Sniffer 模式（`/oemkey=Virtual`）可能使用**硬编码**的连接字符串（如 `FTS4BT Live Import.`），而不是从 `liveimport.ini` 读取
- 如果 FTS 和 DataSource 使用不同的连接字符串，它们永远不会连接到同一个共享内存，所有 API 调用都变成"本地空操作"（hr=0 但无实际作用）

#### 6. `SendNotification` 和 `CheckForMessages` 在 x64 DLL 中存在
测试 v3 确认这两个函数在 `LiveImportAPI_x64.dll` 中导出且调用成功（hr=0x0）。

#### 7. `SendFrame3` 和 `ConvertFiletimeToUnixEpochNs` 在 x64 DLL 中**不存在**
这两个函数（带 Unix 纪元纳秒时间戳）是可选导出，x64 DLL 未包含。

#### 8. `InitializeLiveImportEx` 带 `EQueueMode` 也存在
```c
typedef HRESULT(InitializeLiveImportEx)(
    const TCHAR* szMemoryName,
    const TCHAR* szConfiguration,
    bool* pboolSuccess,
    EQueueMode eQueueMode);  // QUEUE_MODE_DISCARD_QUEUE_CONTENTS_UPON_OVERFLOW = 0
```

### API 头文件确认的完整函数签名

```c
// 基本 5 函数（确认在 x64 DLL 中）
HRESULT InitializeLiveImport(const TCHAR* conn, const TCHAR* config, bool* success);
HRESULT InitializeLiveImportEx(const TCHAR* conn, const TCHAR* config, bool* success, int queueMode);
HRESULT ReleaseLiveImport();
HRESULT IsAppReady(bool* ready);
HRESULT SendFrame(int origLen, int inclLen, const BYTE* data, int drf, int side, int64_t ts100ns);
HRESULT SetExePath(TCHAR* path);  // 注意：不是 const，不是 BSTR

// 额外确认在 x64 DLL 中的函数
HRESULT SendNotification(int eType);        // eStartCaptureToFile=19, eStartCaptureToBuf=18
HRESULT CheckForMessages();
HRESULT StillAlive(bool* alive);
HRESULT GetWorkingDir(TCHAR** dir, int* size);

// 不在 x64 DLL 中的函数
HRESULT SendFrame2(... iDatastreamId ...);   // 不在
HRESULT SendFrame3(... 1ns since 1970 ...);  // 不在
HRESULT ConvertFiletimeToUnixEpochNs(...);   // 不在
```

### drf 值确认
| HCI 类型 | drf 值 | 说明 |
|----------|--------|------|
| Command (0x01) | 1 | 1 << (1-1) |
| ACL Data (0x02) | 2 | 1 << (2-1) |
| SCO Data (0x03) | 4 | 1 << (3-1) |
| Event (0x04) | 8 | 1 << (4-1) |

### 最新测试结果（v3）
```
FTS 启动: OK
InitializeLiveImportEx(QueueMode=Discard): hr=0x0, success=1
SetExePath: hr=0x0
SendNotification(eStartCaptureToFile): hr=0x0
SendNotification(eStartCaptureToBuf): hr=0x0
CheckForMessages: hr=0x0
IsAppReady 60秒: 始终 =0 ← 核心问题
SendFrame 全部: hr=0x0
```

## 最可能的根因

**连接字符串不匹配导致 LiveImport 与 FTS 未真正连接。**

1. 启动 FTS: `Fts.exe /ComProbe Protocol Analysis System=Generic /oemkey=Virtual`
2. FTS 可能使用**硬编码**的连接字符串 `FTS4BT Live Import.` 注册共享内存
3. 我们的 DataSource 使用 `liveimport.ini` 中的 `Wireless Protocol Suite Live Import.FDFFFFFF!...`
4. 两者不匹配 → InitializeLiveImport 返回成功（仅本地初始化），但 IsAppReady 永远无法联系到 FTS
5. SendFrame hr=0（帧被本地排队但从未投递到 FTS）

**验证方法**：将连接字符串改为 `FTS4BT Live Import.`（匹配示例），重新测试。

## 下一步建议（按优先级）

1. **尝试简单连接字符串**：将 `ConnectionString` 改为 `FTS4BT Live Import.`（与示例一致）
2. **运行 GUISample.exe**：启动 FTS → 运行 GUISample → 在 FTS 中点击 "Start Capture" → 在 GUISample 中点击 Action 按钮 → 观察 WPS
3. **移除 SetExePath 调用**：示例和 Android 参考都不使用，可能干扰了正常连接
4. **使用 `IsAppReady(bool*)` 声明**：Rust 侧改用 1 字节参数以精确匹配 DLL 期望
5. **检查 FTS 是否支持 Live Import**：`/oemkey=Virtual` 模式可能需要特定配置
6. **反汇编 DLL**：使用 IDA/Ghidra 确认 SetExePath 和 IsAppReady 的内部行为

## 环境信息
```
WPS 根目录:     C:\software\WPS\
DLL 路径:       C:\software\WPS\Executables\Core\LiveImportAPI_x64.dll
FTS 路径:       C:\software\WPS\Executables\Core\Fts.exe
INI 路径:       C:\software\WPS\liveimport.ini
开发套件:       C:\software\WPS\Live Import Developers Kit\
官方示例源码:   C:\software\WPS\Live Import Developers Kit\h\LiveImportAPI.h
官方 C 示例:    C:\software\WPS\Live Import Developers Kit\Straight C Sample\csample.c
官方 GUI 示例:  C:\software\WPS\Live Import Developers Kit\GUI Sample\maindialog.cpp
示例 INI:       C:\software\WPS\Live Import Developers Kit\liveimport.ini
PDF 手册:       C:\project\Logzilla\Logzilla\live_import_ref.pdf
测试程序:       C:\Users\jamin\AppData\Local\Temp\opencode\live_import_test\src\main.rs
主工程:         C:\project\Logzilla\Logzilla\src-tauri\src\commands\live_import_commands.rs
```

## 参考链接
- Android AOSP btsnoop_live.py: https://android.googlesource.com/platform/packages/modules/Bluetooth/+/refs/heads/main/system/tools/scripts/btsnoop_live.py
- Teledyne LeCroy: http://www.fte.com

## 本文件来源
- 由 opencode（AI 编码助手）在 2026-05-23 生成，第二次更新
- 用于交接给下一个 AI 继续排查 Live Import 问题
