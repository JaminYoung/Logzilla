# Logzilla 边界问题修复方案

本文档记录当前代码审查发现的软件边界问题和建议修复方案。此文档只描述方案，不代表代码已经修复。

## 目标

- 降低 Tauri WebView 被异常输入或前端漏洞放大为本机文件读写风险的可能性。
- 避免畸形日志、DCF、pcap/pcapng、LC3 输入导致后端 panic。
- 给长时间串口采集、大文件导入、IPC 数据回传增加硬边界。
- 让测试套件可以在干净仓库和 CI 中稳定运行。

## 优先级总览

| 优先级 | 问题 | 风险 | 建议状态 |
| --- | --- | --- | --- |
| P0 | Tauri 文件权限和后端任意路径读写过宽 | 本机文件读写面过大 | 优先修 |
| P0 | HCI、DCF、pcapng 解析可 panic | 用户输入文件可导致命令崩溃 | 优先修 |
| P1 | 串口、LC3 capture、大文件解析无硬上限 | 长时间运行或大文件导致内存耗尽 | 优先修 |
| P1 | LC3 导出路径直接拼接 `base_name` | 输出目录穿透 | 优先修 |
| P2 | LiveImport 递归搜索和 DLL 加载边界宽 | 慢扫描、误加载、非预期目录 | 规划修 |
| P2 | 测试依赖仓库外 `../../app.dcf` | CI 不稳定 | 规划修 |
| P3 | 中文字符串乱码 | 用户提示和测试断言可读性差 | 顺手修 |

## 1. 收紧文件系统边界

### 现状

- `src-tauri/capabilities/default.json` 开启了宽泛的 `fs:allow-read`、`fs:allow-write`、`fs:allow-remove`、`fs:allow-rename`、`fs:allow-copy-file`、`fs:allow-read-dir`。
- `src-tauri/src/commands/file_commands.rs` 暴露 `read_file(path)` 和 `write_file(path, content)`，后端直接使用调用方传入路径。
- `src-tauri/tauri.conf.json` 中 `csp` 为 `null`，前端一旦出现 XSS 或加载异常内容，风险会被文件权限放大。

### 修复方案

1. 删除默认能力里的通用 `fs:*` 权限，只保留确实需要的 `dialog:*` 权限。
2. 后端读写命令不要接受任意路径。建议拆成用途明确的命令：
   - 保存日志：只允许写入用户通过保存对话框选择的目标文件。
   - 导入规则 JSON：只允许读用户通过打开对话框选择的 JSON 文件，并限制大小。
   - 导出规则 JSON：只允许写用户选择的目标文件。
3. 对所有路径做规范化校验：
   - 使用 `PathBuf`，不要字符串拼接路径。
   - 拒绝空路径、相对路径、包含 `..` 的输出文件名、包含路径分隔符的 `base_name`。
   - 对目录输出执行 `canonicalize`，写入前确认最终路径仍在目标目录内。
4. 给 `read_file` 设置大小限制，例如规则文件最大 1 MB，日志/导入文件按功能分别限制。
5. 配置 CSP。初始可从较保守策略开始，例如只允许 `self`、必要的 wasm/script/style 来源，避免继续使用 `csp: null`。

### 验收

- 前端仍能导入/导出过滤规则、导出高亮规则、保存日志。
- 任意构造 `invoke('write_file', { path: 'C:\\Windows\\...' })` 不再能直接写入。
- 传入 `..\\evil`、绝对路径文件名、带 `/` 或 `\` 的 `base_name` 会返回错误。

## 2. 修复解析器 panic

### HCI 时间戳解析

现状：

- `parse_hci_line` 只检查时间戳长度和 ASCII。
- `local_pc_timestamp_to_btsnoop`、`chip_timestamp_to_btsnoop` 对 `NaiveTime::from_hms_milli_opt(...).unwrap()`。
- 畸形时间如 `99:99:99.999` 会 panic。

修复：

1. 增加 `parse_hms_millis(ts: &str) -> Option<(u32, u32, u32, u32)>`。
2. 校验格式必须是 `HH:MM:SS.mmm`，第 2、5、8 位分隔符固定。
3. 使用 `NaiveTime::from_hms_milli_opt` 返回 `Option`，不要 unwrap。
4. 非法行跳过，或让 `extract_hci` 返回清晰错误，不能 panic。

测试：

- `(99:99:99.999) CMD => 03 0c 00` 不 panic。
- `[00:00:99.000] CMD => 03 0c 00` 不 panic。
- 正常 HCI 样例仍能生成 `.cfa`。

### DCF / XCFG 越界

现状：

- `parse_dcf` 验证了 `info_offset + 16 <= data.len()`。
- `parse_full_xcfg` 直接切片 `data[info_offset + 16..info_offset + 16 + info_len]`。
- 畸形 DCF 中 `info_len` 过大时会 panic。

修复：

1. 在 `parse_full_xcfg` 中使用 `checked_add` 计算 `info_start`、`info_end`。
2. 校验 `info_end <= data.len()`，失败返回 `anyhow!("INFO data extends beyond DCF file")`。
3. `parse_dcf` 也建议校验 `data_size` 与实际文件大小关系，至少记录或返回格式错误。

测试：

- 构造 `INFO` header 有效但 `info_len` 超出文件长度的 DCF，`open_dcf` 返回 Err，不 panic。

### pcapng Simple Packet Block

现状：

- `read_pcapng` 对 block type `0x00000003` 直接使用 `offset + 24..offset + 24 + orig_len`。
- 没有按 `block_len` 校验数据区边界。
- Simple Packet Block 结构也不应按当前偏移解析。

修复：

1. 按 pcapng 规范重新解析 SPB：
   - block header 8 bytes。
   - original packet length 在 block body 开始处。
   - packet data 从 `offset + 12` 开始。
   - packet data 长度应使用 `min(original_len, captured_len)` 的可验证长度，不能超过 `block_len - 16`。
2. 所有 `offset + n` 使用 `checked_add` 或先验证边界。
3. 对未知或损坏 block 返回错误或跳过，但不要 panic。

测试：

- block_len 很小、orig_len 很大时返回 Err 或跳过，不 panic。
- 正常 pcapng 仍能分析 USB audio。

## 3. 增加内存和输入大小上限

### 串口日志

现状：

- `line_buf` 在串口持续输出但没有换行时会无限增长。
- `max_lines` 由前端传入，后端没有上限。

修复：

1. 定义后端常量：
   - `MAX_LOG_LINES`，例如 200_000。
   - `MAX_LINE_BYTES`，例如 64 KiB。
2. `start_serial_reader` 接收的 `max_lines` 使用 `clamp(500, MAX_LOG_LINES)`。
3. `line_buf` 超过 `MAX_LINE_BYTES` 时截断并生成一条提示，或强制 flush 当前行。

测试：

- 模拟没有换行的 1 MB 串口数据，后端内存不会持续增长。
- 传 `max_lines = usize::MAX` 不会造成异常分配或长期 drain。

### LC3 capture

现状：

- capture 数据在 `lc3_capture_buffers` 中持续累积。
- 停止 capture 时通过 IPC 一次性返回整个 `Vec<u8>` 到前端。

修复：

1. 定义 `MAX_LC3_CAPTURE_BYTES`，例如 100 MB 或根据业务决定。
2. 超过限制时自动停止 capture，状态中返回 `truncated: true` 或错误提示。
3. 长期方案：capture 直接写入临时文件或用户选择的输出文件，前端只拿路径和统计信息，不通过 IPC 传整块数据。

测试：

- 超过上限时不会继续扩容。
- 停止 capture 后不会通过 IPC 返回超大数组。

### 大文件导入与解析

现状：

- HCI、DCF、pcap、LC3 导入多处 `std::fs::read` 或 `read_to_string` 一次性读入。
- LC3 文本导入 `read_first_line` 也会读完整文件。

修复：

1. 打开文件前先读取 metadata，根据功能限制大小。
2. 文本首行读取改用 `BufRead::read_line`。
3. HCI 提取改为 `BufReader` 逐行解析，直接写 `.cfa`，避免缓存全部 `packets`。
4. pcap/pcapng 可逐包迭代，USB audio extract 如果需要分段缓存，也要给包数和总数据量设上限。

测试：

- 超过大小限制时返回明确错误。
- 1 GB 文本文件不会被一次性加载。

## 4. 修复 LC3 导出路径和参数边界

### 路径

现状：

- `lc3_decode_and_export` 使用 `format!("{}\\{}.wav", output_dir, base_name)`。
- `base_name` 未校验。

修复：

1. 增加 `sanitize_file_stem(base_name: &str) -> Result<String, String>`。
2. 只允许文件名字符，例如字母、数字、空格、`_`、`-`、`.`，拒绝 `\`、`/`、`:`、`..`。
3. 使用 `PathBuf::from(output_dir).join(format!("{stem}.wav"))`。
4. 写入前确认输出目录存在且是目录。

### 参数

现状：

- `sample_rate`、`num_channels`、`bitrate`、`frame_duration_ms` 直接传给 LC3 FFI。

修复：

1. 在后端定义白名单：
   - sample rate: 8000、16000、24000、32000、44100、48000、96000 等业务支持值。
   - channels: 1 或 2，除非确认库支持更多。
   - frame duration: 2.5、5、7.5、10 ms 等业务支持值。
   - bitrate: 设置合理上下限。
2. FFI 返回前后都不要 unwrap，不可信参数统一返回 Err。

测试：

- `num_channels = 0`、超大 `num_channels`、`frame_duration_ms = NaN/Infinity`、`base_name = ..\\x` 都返回 Err。

## 5. LiveImport 边界

现状：

- `find_named_file_recursive` 会递归扫描用户传入目录。
- 32 位构建会从该目录下查找并加载 `LiveImportAPI.dll`。
- `live_import_send_frame` 中 `h4_type` 会参与位移表达式，应限制取值。

修复：

1. 限制 `wps_path` 必须是目录，且优先只检查已知固定相对路径。
2. 若要递归搜索，限制最大深度和最大访问目录数。
3. DLL 加载前确认文件名和所在目录符合预期。
4. `h4_type` 只允许 `1`、`2`、`4`，其他值直接返回 Err。

测试：

- 传入根目录或超深目录不会长时间卡住。
- `h4_type = 0`、`255` 不会 panic 或产生无意义位移。

## 6. 测试和 CI

现状：

- `cargo test --manifest-path src-tauri/Cargo.toml --target i686-pc-windows-msvc` 当前结果为 19 passed、1 failed。
- 失败测试是 `tests::test_dcf_parse`，硬编码读取 `../../app.dcf`，仓库内没有该文件。

修复：

1. 将 DCF 样例放到 `src-tauri/tests/fixtures/`，使用相对 `CARGO_MANIFEST_DIR` 的稳定路径。
2. 如果样例文件不能提交，将测试标记为 `#[ignore]`，或缺文件时跳过而不是 `expect`。
3. 给上述边界问题补充单元测试。
4. 建议 CI 至少执行：
   - `npm.cmd run build` 或跨平台 `npm run build`。
   - `cargo test --manifest-path src-tauri/Cargo.toml`。
   - Windows 发布目标下的 `cargo test --target i686-pc-windows-msvc`。

## 7. 建议实施顺序

1. 先修 panic：HCI 时间戳、DCF/XCFG 越界、pcapng 越界。
2. 再收紧文件系统权限和路径校验。
3. 加内存上限：串口行长、日志行数、LC3 capture、文件大小。
4. 修 LC3 导出路径和参数白名单。
5. 修测试 fixture，让 CI 可以稳定跑。
6. 最后处理中文乱码和错误提示文案。

## 当前验证记录

- `npm.cmd run build`：通过。
- `npm run build`：在当前 PowerShell 环境被执行策略拦截，原因是 `npm.ps1` 禁止执行，不是项目 TypeScript/Vite 错误。
- `cargo test --manifest-path src-tauri\Cargo.toml --target i686-pc-windows-msvc`：失败，原因是 `tests::test_dcf_parse` 找不到 `../../app.dcf`。

