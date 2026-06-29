# 修复：大量日志涌入时卡死 / 到缓冲上限后不再显示

本补丁修复两个独立 bug：
  Bug 1（卡死）：前端日志拉取 useEffect 依赖 logsVersion，每批新日志都销毁/
                重建 effect，堆叠出大量并发 setTimeout 轮询链，主线程被 invoke
                淹没而卡死；且闭包捕获陈旧 logs.length 导致漏拉/重复。
  Bug 2（不显示）：后端缓冲超过 max_lines（默认 500）从头 drain 后，前端累计
                  行号 knownLines 与缓冲下标错位，get_logs 永远返回空，日志到
                  500 行后彻底停住。

修复要点：
  - 后端新增绝对游标 logs_base/logs2_base（记录已丢弃行数），get_logs 用
    `known_lines - base` 映射缓冲区内下标；裁剪只增 base。
  - 前端 version 与已知行数改用 ref 跟踪，轮询 effect 不再依赖 version，
    循环只建一次、稳定运行。

涉及文件：
  src-tauri/src/lib.rs
  src-tauri/src/commands/serial_commands.rs
  src-tauri/src/commands/search_commands.rs
  src/App.tsx

================================================================================

--- a/src-tauri/src/lib.rs
+++ b/src-tauri/src/lib.rs
@@ AppState 结构体字段
     pub logs: Mutex<Vec<String>>,
     pub logs2: Mutex<Vec<String>>,
     pub logs_version: Mutex<u64>,
     pub logs2_version: Mutex<u64>,
+    /// 已从缓冲区头部丢弃的行数（绝对游标基准）。前端用累计行号请求，
+    /// 后端用 `绝对行号 - base` 定位到当前缓冲区内的实际下标，避免裁剪后索引错乱。
+    pub logs_base: Mutex<usize>,
+    pub logs2_base: Mutex<usize>,
     /// 每个串口的不完整行缓冲（跨次读取拼接）
     pub partial_lines: Mutex<HashMap<String, String>>,
 }
@@ impl Default for AppState
             logs: Mutex::new(Vec::new()),
             logs2: Mutex::new(Vec::new()),
             logs_version: Mutex::new(0),
             logs2_version: Mutex::new(0),
+            logs_base: Mutex::new(0),
+            logs2_base: Mutex::new(0),
             partial_lines: Mutex::new(HashMap::new()),


--- a/src-tauri/src/commands/serial_commands.rs
+++ b/src-tauri/src/commands/serial_commands.rs
@@ read_serial_data: 写入缓冲后限制最大行数
             logs.extend(new_lines);

-            // 限制最大行数
-            if logs.len() > max_lines {
-                let drain_count = logs.len() - max_lines;
-                logs.drain(0..drain_count);
-            }
+            // 限制最大行数：从头丢弃旧行，并把丢弃数累加到 base 游标，
+            // 使前端的累计行号仍能正确映射到缓冲区内下标。
+            if logs.len() > max_lines {
+                let drain_count = logs.len() - max_lines;
+                logs.drain(0..drain_count);
+                let mut base = if panel == 1 {
+                    state.logs2_base.lock().map_err(|e| e.to_string())?
+                } else {
+                    state.logs_base.lock().map_err(|e| e.to_string())?
+                };
+                *base += drain_count;
+            }

             // 更新版本号
             *version += 1;


--- a/src-tauri/src/commands/search_commands.rs
+++ b/src-tauri/src/commands/search_commands.rs
@@ get_logs: 计算返回的新增行
     // 如果指定了版本号，且当前版本号未变化，返回空
     if let Some(since) = since_version {
         if *version <= since {
             return Ok((vec![], *version));
         }
     }

-    // 只返回前端尚未拥有的新增行
-    let start = known_lines.unwrap_or(0).min(logs.len());
-    let new_lines = logs[start..].to_vec();
+    // base = 已丢弃的行数；前端传来的 known_lines 是累计行号。
+    // 用 known_lines - base 得到缓冲区内的起始下标；若前端落后于已丢弃位置，则从头返回。
+    let base = if panel == 0 {
+        *state.logs_base.lock().map_err(|e| e.to_string())?
+    } else {
+        *state.logs2_base.lock().map_err(|e| e.to_string())?
+    };
+    let known = known_lines.unwrap_or(0);
+    let start = known.saturating_sub(base).min(logs.len());
+    let new_lines = logs[start..].to_vec();

     Ok((new_lines, *version))
 }
@@ clear_logs: 清空时重置 base 游标
     if panel == 0 {
         state.logs.lock().map_err(|e| e.to_string())?.clear();
+        *state.logs_base.lock().map_err(|e| e.to_string())? = 0;
         *state.logs_version.lock().map_err(|e| e.to_string())? += 1;
     } else {
         state.logs2.lock().map_err(|e| e.to_string())?.clear();
+        *state.logs2_base.lock().map_err(|e| e.to_string())? = 0;
         *state.logs2_version.lock().map_err(|e| e.to_string())? += 1;
     }
     Ok(())


--- a/src/App.tsx
+++ b/src/App.tsx
@@ 状态声明：删除 logsVersion/logs2Version 两个 useState，改用 ref
-  const [logsVersion, setLogsVersion] = useState(0);
-  const [logs2Version, setLogs2Version] = useState(0);
+  // 日志版本号 / 已知行数游标：用 ref 跟踪，避免轮询 effect 随之销毁重建
+  // （旧实现用 useState 会堆叠并发轮询链并因闭包陈旧值丢日志）。
+  const logsVersionRef = useRef(0);
+  const logs2VersionRef = useRef(0);
+  const logsLenRef = useRef(0);
+  const logs2LenRef = useRef(0);
@@ 新增统一清空处理函数（放在 showToast 之后）
+  // 清空日志：同时重置前端显示、行号/版本游标与后端缓冲，保持两端同步
+  const handleClearLogs = async (panel: 0 | 1) => {
+    try {
+      const { invoke } = await import('@tauri-apps/api/core');
+      await invoke('clear_logs', { panel });
+    } catch (e) {
+      console.error('Clear logs error:', e);
+    }
+    if (panel === 0) {
+      logsLenRef.current = 0;
+      logsVersionRef.current = 0;
+      setLogs([]);
+    } else {
+      logs2LenRef.current = 0;
+      logs2VersionRef.current = 0;
+      setLogs2([]);
+    }
+  };
@@ 面板1 日志拉取 effect（整体替换）
-  useEffect(() => {
-    let isActive = true;
-
-    const fetchLogs = async () => {
-      if (!isActive) return;
-      try {
-        const { invoke } = await import('@tauri-apps/api/core');
-        const [newLogs, newVersion] = await invoke<[string[], number]>('get_logs', {
-          panel: 0,
-          sinceVersion: logsVersion,
-          knownLines: logs.length
-        });
-
-        if (newVersion > logsVersion && newLogs.length > 0) {
-          setLogs(prev => [...prev, ...newLogs]);
-          setLogsVersion(newVersion);
-        } else if (newVersion > logsVersion) {
-          setLogsVersion(newVersion);
-        }
-      } catch (e) {
-        console.error('Fetch logs error:', e);
-      }
-
-      if (isActive) {
-        setTimeout(fetchLogs, pollIntervalMs);
-      }
-    };
-
-    fetchLogs();
-    return () => { isActive = false; };
-  }, [logsVersion, pollIntervalMs]);
+  useEffect(() => {
+    let isActive = true;
+    let timer: ReturnType<typeof setTimeout> | null = null;
+
+    const fetchLogs = async () => {
+      if (!isActive) return;
+      try {
+        const { invoke } = await import('@tauri-apps/api/core');
+        const [newLogs, newVersion] = await invoke<[string[], number]>('get_logs', {
+          panel: 0,
+          sinceVersion: logsVersionRef.current,
+          knownLines: logsLenRef.current
+        });
+
+        if (newVersion > logsVersionRef.current) {
+          logsVersionRef.current = newVersion;
+          if (newLogs.length > 0) {
+            logsLenRef.current += newLogs.length;
+            setLogs(prev => [...prev, ...newLogs]);
+          }
+        }
+      } catch (e) {
+        console.error('Fetch logs error:', e);
+      }
+
+      if (isActive) {
+        timer = setTimeout(fetchLogs, pollIntervalMs);
+      }
+    };
+
+    fetchLogs();
+    return () => { isActive = false; if (timer) clearTimeout(timer); };
+  }, [pollIntervalMs]);
@@ 面板2 日志拉取 effect（整体替换）
-  useEffect(() => {
-    if (!dualPanelMode) return;
-    let isActive = true;
-
-    const fetchLogs2 = async () => {
-      if (!isActive) return;
-      try {
-        const { invoke } = await import('@tauri-apps/api/core');
-        const [newLogs, newVersion] = await invoke<[string[], number]>('get_logs', {
-          panel: 1,
-          sinceVersion: logs2Version,
-          knownLines: logs2.length
-        });
-
-        if (newVersion > logs2Version && newLogs.length > 0) {
-          setLogs2(prev => [...prev, ...newLogs]);
-          setLogs2Version(newVersion);
-        } else if (newVersion > logs2Version) {
-          setLogs2Version(newVersion);
-        }
-      } catch (e) {
-        console.error('Fetch logs2 error:', e);
-      }
-
-      if (isActive) {
-        setTimeout(fetchLogs2, pollIntervalMs);
-      }
-    };
-
-    fetchLogs2();
-    return () => { isActive = false; };
-  }, [dualPanelMode, logs2Version, pollIntervalMs]);
+  useEffect(() => {
+    if (!dualPanelMode) return;
+    let isActive = true;
+    let timer: ReturnType<typeof setTimeout> | null = null;
+
+    const fetchLogs2 = async () => {
+      if (!isActive) return;
+      try {
+        const { invoke } = await import('@tauri-apps/api/core');
+        const [newLogs, newVersion] = await invoke<[string[], number]>('get_logs', {
+          panel: 1,
+          sinceVersion: logs2VersionRef.current,
+          knownLines: logs2LenRef.current
+        });
+
+        if (newVersion > logs2VersionRef.current) {
+          logs2VersionRef.current = newVersion;
+          if (newLogs.length > 0) {
+            logs2LenRef.current += newLogs.length;
+            setLogs2(prev => [...prev, ...newLogs]);
+          }
+        }
+      } catch (e) {
+        console.error('Fetch logs2 error:', e);
+      }
+
+      if (isActive) {
+        timer = setTimeout(fetchLogs2, pollIntervalMs);
+      }
+    };
+
+    fetchLogs2();
+    return () => { isActive = false; if (timer) clearTimeout(timer); };
+  }, [dualPanelMode, pollIntervalMs]);
@@ 三处清空按钮 onClear：由直接 setLogs/setLogs2 改为调用 handleClearLogs
-              onClear={() => setLogs([])}      // 面板1（单面板模式）
+              onClear={() => handleClearLogs(0)}
   ...
-                onClear={() => setLogs([])}    // 面板1（双面板模式）
+                onClear={() => handleClearLogs(0)}
   ...
-                onClear={() => setLogs2([])}   // 面板2（双面板模式）
+                onClear={() => handleClearLogs(1)}

================================================================================
备注：
  - App.tsx 中凡是引用 logsVersion / logs2Version / setLogsVersion /
    setLogs2Version 的地方都要删除（已被 ref 取代，UI 不再读这两个 state）。
  - 关键不变量：① 后端用绝对游标 base 做行号映射，裁剪只增 base；
    ② 前端轮询 effect 依赖数组只含 [pollIntervalMs]（面板2加 dualPanelMode），
    绝不依赖 version，否则又会堆叠重建导致卡死。