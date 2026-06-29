use std::sync::Mutex;

// ========= Shared utilities (always compiled) =========

#[allow(dead_code)]
type HRESULT = i32;
#[allow(dead_code)]
const S_OK: HRESULT = 0;

fn pc_timestamp_to_filetime(ts: &str) -> i64 {
    let ts = ts.trim();
    if ts.len() < 12 { return current_filetime(); }
    let h: u32 = match ts[0..2].parse() { Ok(v) => v, Err(_) => return current_filetime() };
    let m: u32 = match ts[3..5].parse() { Ok(v) => v, Err(_) => return current_filetime() };
    let s: u32 = match ts[6..8].parse() { Ok(v) => v, Err(_) => return current_filetime() };
    let ms: u32 = match ts[9..12].parse() { Ok(v) => v, Err(_) => return current_filetime() };
    let now_local = chrono::Local::now();
    let date = now_local.date_naive();
    let local_naive = chrono::NaiveDateTime::new(date, chrono::NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap());
    let utc_dt = local_naive.and_local_timezone(chrono::Local).unwrap().with_timezone(&chrono::Utc);
    unix_usec_to_filetime(utc_dt.timestamp_micros() as i64)
}

fn current_filetime() -> i64 {
    unix_usec_to_filetime(chrono::Utc::now().timestamp_micros())
}

fn unix_usec_to_filetime(unix_usec: i64) -> i64 {
    let unix_sec = unix_usec / 1_000_000;
    let sub_us = unix_usec % 1_000_000;
    (unix_sec + 11644473600) * 10_000_000 + sub_us * 10
}

fn parse_ini(path: &str) -> Result<(String, String), String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取 INI 失败: {}", e))?;
    let mut connection_string = String::new();
    let mut config_lines: Vec<String> = Vec::new();
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_config = trimmed.eq_ignore_ascii_case("[configuration]");
            continue;
        }
        if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() { continue; }
        if trimmed.starts_with("ConnectionString=") {
            connection_string = trimmed["ConnectionString=".len()..].to_string();
        } else if in_config {
            config_lines.push(trimmed.to_string());
        }
    }
    if connection_string.is_empty() { return Err("INI 文件中未找到 ConnectionString".to_string()); }
    let config = config_lines.join("\n") + "\n";
    Ok((connection_string, config))
}

fn find_wps_exe(wps_path: &str) -> Option<String> {
    let path = std::path::Path::new(wps_path).join("Executables").join("Core").join("Fts.exe");
    if path.exists() { Some(path.to_string_lossy().to_string()) } else { None }
}

#[derive(serde::Serialize)]
pub struct LiveImportDiag {
    pub conn_string: String,
    pub dll_loaded: bool,
    pub init_hr: i32,
    pub init_success: i32,
    pub fts_running: bool,
}

fn is_fts_running() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("tasklist")
            .args(["/fi", "imagename eq Fts.exe", "/nh"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("Fts.exe"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[allow(dead_code)]
fn to_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[derive(serde::Serialize, Default)]
pub struct SendStats {
    pub total: u64,
    pub ok: u64,
    pub err: u64,
    pub last_hr: i32,
    pub last_err_msg: String,
}

static SEND_STATS: Mutex<SendStats> = Mutex::new(SendStats { total: 0, ok: 0, err: 0, last_hr: 0, last_err_msg: String::new() });

fn inc_send_ok() {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.ok += 1;
    s.last_hr = 0;
}

fn inc_send_err(hr: i32, msg: String) {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.err += 1;
    s.last_hr = hr;
    s.last_err_msg = msg.clone();
    eprintln!("[LOGZILLA] SendFrame ERR: 0x{:x} {}", hr, msg);
}

// ========= 64-bit: Helper process implementation =========
#[cfg(target_pointer_width = "64")]
mod imp {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    struct Helper {
        process: std::process::Child,
        stdin: std::process::ChildStdin,
        stdout: BufReader<std::process::ChildStdout>,
    }
    unsafe impl Send for Helper {}
    unsafe impl Sync for Helper {}

    impl Helper {
        fn send_cmd(&mut self, cmd: &str) -> Result<String, String> {
            writeln!(self.stdin, "{}", cmd).map_err(|e| format!("写入命令失败: {}", e))?;
            self.stdin.flush().map_err(|e| format!("刷新失败: {}", e))?;
            let mut line = String::new();
            self.stdout.read_line(&mut line).map_err(|e| format!("读取响应失败: {}", e))?;
            Ok(line.trim().to_string())
        }
        fn init(&mut self, conn: &str, config: &str) -> Result<(), String> {
            let config = config.replace('\n', "\x1E");
            let r = self.send_cmd(&format!("INIT|{}|{}", conn, config))?;
            if r == "OK" { Ok(()) } else { Err(r.trim_start_matches("ERR|").to_string()) }
        }
        fn is_ready(&mut self) -> Result<bool, String> {
            let r = self.send_cmd("READY")?;
            if r.starts_with("OK|") { Ok(r.trim_start_matches("OK|") == "1") } else { Err(r.trim_start_matches("ERR|").to_string()) }
        }
        fn send_frame(&mut self, drf: i32, side: i32, ts: i64, hex: &str) -> Result<(), String> {
            let r = self.send_cmd(&format!("FRAME|{}|{}|{}|{}", drf, side, ts, hex))?;
            if r.starts_with("OK") { 
                inc_send_ok();
                Ok(()) 
            } else { 
                let msg = r.trim_start_matches("ERR|").to_string();
                inc_send_err(-1, msg.clone());
                Err(msg.clone())
            }
        }
        fn close(mut self) {
            let _ = writeln!(self.stdin, "CLOSE"); let _ = self.stdin.flush(); let _ = self.process.wait();
        }
    }

    fn helper_exe() -> String {
        std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("live_import_helper.exe").to_string_lossy().to_string()
    }

    static LIVE_IMPORT: Mutex<Option<Helper>> = Mutex::new(None);

    pub fn init(wps_path: &str) -> Result<LiveImportDiag, String> {
        let ini_path = std::path::Path::new(wps_path).join("liveimport.ini");
        let (conn_str, config_str) = parse_ini(ini_path.to_str().unwrap())?;
        let fts_running = is_fts_running();
        let helper_path = helper_exe();
        if !std::path::Path::new(&helper_path).exists() {
            return Ok(LiveImportDiag { conn_string: conn_str, dll_loaded: false, init_hr: -1, init_success: 0, fts_running });
        }
        let mut child = match Command::new(&helper_path).arg(wps_path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn() {
            Ok(c) => c,
            Err(_) => return Ok(LiveImportDiag { conn_string: conn_str, dll_loaded: false, init_hr: -1, init_success: 0, fts_running }),
        };
        let stdin = child.stdin.take().ok_or("无法获取 stdin")?;
        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let mut helper = Helper { process: child, stdin, stdout: BufReader::new(stdout) };
        let r = helper.init(&conn_str, &config_str);
        let (hr, success) = match &r { Ok(()) => (0, 1), Err(_) => (-1, 0) };
        let diag = LiveImportDiag { conn_string: conn_str, dll_loaded: true, init_hr: hr, init_success: success, fts_running };
        if r.is_ok() { let mut g = LIVE_IMPORT.lock().unwrap(); *g = Some(helper); }
        Ok(diag)
    }

    pub fn send_frame(data_hex: &str, h4_type: u8, sent: bool, pc_timestamp: &str) -> Result<(), String> {
        let mut g = LIVE_IMPORT.lock().unwrap();
        let helper = g.as_mut().ok_or("Live Import 未初始化")?;
        let drf: i32 = 1 << ((h4_type as i32) - 1);
        let side: i32 = if sent { 0 } else { 1 };
        let ts = if pc_timestamp.is_empty() { current_filetime() } else { pc_timestamp_to_filetime(pc_timestamp) };
        let r = helper.send_frame(drf, side, ts, data_hex);
        eprintln!("[LOGZILLA] send_frame drf={drf} side={side} hex={data_hex} -> {:?}", r);
        r
    }

    pub fn is_ready() -> Result<bool, String> {
        let mut g = LIVE_IMPORT.lock().unwrap();
        g.as_mut().ok_or("Live Import 未初始化")?.is_ready()
    }

    pub fn close() {
        let mut g = LIVE_IMPORT.lock().unwrap();
        if let Some(h) = g.take() {
            eprintln!("[LOGZILLA] live_import_close: stopping helper process");
            h.close();
        } else {
            eprintln!("[LOGZILLA] live_import_close: nothing to close (64-bit)");
        }
        let mut s = SEND_STATS.lock().unwrap();
        *s = SendStats::default();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// ========= 32-bit: Direct DLL implementation =========
#[cfg(target_pointer_width = "32")]
mod imp {
    use super::*;
    use libloading::Library;
    use std::ffi::CString;
    #[link(name = "kernel32")]
    extern "system" { fn SetDllDirectoryW(lpPathName: *const u16) -> i32; }

    struct DirectDll {
        _lib: Library,
        release: unsafe extern "C" fn() -> HRESULT,
        is_app_ready: unsafe extern "C" fn(*mut i32) -> HRESULT,
        send_frame: unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> HRESULT,
    }
    unsafe impl Send for DirectDll {}
    unsafe impl Sync for DirectDll {}

    static LIVE_IMPORT: Mutex<Option<DirectDll>> = Mutex::new(None);

    pub fn init(wps_path: &str) -> Result<LiveImportDiag, String> {
        let ini_path = std::path::Path::new(wps_path).join("liveimport.ini");
        let (conn_str, config_str) = parse_ini(ini_path.to_str().unwrap())?;
        let fts_running = is_fts_running();

        let dll_path = std::path::Path::new(wps_path).join("Executables").join("Core").join("LiveImportAPI.dll");
        if !dll_path.exists() {
            return Ok(LiveImportDiag { conn_string: conn_str, dll_loaded: false, init_hr: -1, init_success: 0, fts_running });
        }

        let dll_path_str = dll_path.to_string_lossy().to_string();
        let core_dir = std::path::Path::new(wps_path).join("Executables").join("Core");
        let core_dir_wide = to_utf16(&core_dir.to_string_lossy());
        unsafe { SetDllDirectoryW(core_dir_wide.as_ptr()); }

        let lib = match unsafe { Library::new(&dll_path_str) } {
            Ok(l) => { unsafe { SetDllDirectoryW(std::ptr::null()); } l }
            Err(_) => {
                unsafe { SetDllDirectoryW(std::ptr::null()); }
                return Ok(LiveImportDiag { conn_string: conn_str, dll_loaded: false, init_hr: -1, init_success: 0, fts_running });
            }
        };

        macro_rules! load_fn {
            ($name:literal, $ty:ty) => {{
                let name: &[u8] = $name;
                unsafe {
                    let sym: libloading::Symbol<unsafe extern "C" fn()> = lib.get::<unsafe extern "C" fn()>(name)
                        .expect(concat!("函数 ", stringify!($name), " 未找到"));
                    std::mem::transmute::<*const (), $ty>(*sym as *const ())
                }
            }};
        }

        let initialize: unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char, *mut i32) -> HRESULT =
            load_fn!(b"InitializeLiveImport", unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char, *mut i32) -> HRESULT);
        let release: unsafe extern "C" fn() -> HRESULT =
            load_fn!(b"ReleaseLiveImport", unsafe extern "C" fn() -> HRESULT);
        let is_app_ready: unsafe extern "C" fn(*mut i32) -> HRESULT =
            load_fn!(b"IsAppReady", unsafe extern "C" fn(*mut i32) -> HRESULT);
        let send_frame: unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> HRESULT =
            load_fn!(b"SendFrame", unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> HRESULT);

        let conn_cstr = CString::new(conn_str.as_bytes()).map_err(|_| "连接字符串无效".to_string())?;
        let config_cstr = CString::new(config_str.as_bytes()).map_err(|_| "配置字符串无效".to_string())?;

        let mut success: i32 = 0;
        let hr = unsafe { initialize(conn_cstr.as_ptr(), config_cstr.as_ptr(), &mut success) };

        let dll = DirectDll { _lib: lib, release, is_app_ready, send_frame };

        if hr == S_OK && success != 0 {
            let mut g = LIVE_IMPORT.lock().unwrap();
            *g = Some(dll);
        }

        Ok(LiveImportDiag { conn_string: conn_str, dll_loaded: true, init_hr: hr, init_success: success, fts_running })
    }

    pub fn send_frame(data_hex: &str, h4_type: u8, sent: bool, pc_timestamp: &str) -> Result<(), String> {
        let g = LIVE_IMPORT.lock().unwrap();
        let dll = g.as_ref().ok_or("Live Import 未初始化")?;
        let data: Vec<u8> = data_hex.split_whitespace().map(|b| u8::from_str_radix(b, 16).map_err(|_| format!("无效十六进制: {}", b))).collect::<Result<Vec<u8>, String>>()?;
        if data.is_empty() { return Err("空数据帧".to_string()); }
        let drf: i32 = 1 << ((h4_type as i32) - 1);
        let side: i32 = if sent { 0 } else { 1 };
        let ts = if pc_timestamp.is_empty() { current_filetime() } else { pc_timestamp_to_filetime(pc_timestamp) };
        let len = data.len() as i32;
        let hr = unsafe { (dll.send_frame)(len, len, data.as_ptr(), drf, side, ts) };
        eprintln!("[LOGZILLA] send_frame drf={drf} side={side} len={len} hr=0x{hr:x}");
        if hr == S_OK { inc_send_ok(); Ok(()) } else { inc_send_err(hr, format!("SendFrame 失败: 0x{:x}", hr)); Err(format!("SendFrame 失败: 0x{:x}", hr)) }
    }

    pub fn is_ready() -> Result<bool, String> {
        let g = LIVE_IMPORT.lock().unwrap();
        let dll = g.as_ref().ok_or("Live Import 未初始化")?;
        let mut ready: i32 = 0;
        let hr = unsafe { (dll.is_app_ready)(&mut ready) };
        if hr == S_OK { Ok(ready != 0) } else { Err(format!("IsAppReady 失败: 0x{:x}", hr)) }
    }

    pub fn close() {
        let mut g = LIVE_IMPORT.lock().unwrap();
        if let Some(dll) = g.take() {
            let hr = unsafe { (dll.release)() };
            eprintln!("[LOGZILLA] live_import_close: ReleaseLiveImport hr=0x{:x}", hr);
        } else {
            eprintln!("[LOGZILLA] live_import_close: nothing to close (32-bit)");
        }
        let mut s = SEND_STATS.lock().unwrap();
        *s = SendStats::default();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// ========= Public Tauri commands =========

#[tauri::command]
pub fn live_import_init(wps_path: String) -> Result<LiveImportDiag, String> {
    imp::init(&wps_path)
}

#[tauri::command]
pub fn live_import_send_frame(data_hex: String, h4_type: u8, sent: bool, pc_timestamp: String) -> Result<(), String> {
    imp::send_frame(&data_hex, h4_type, sent, &pc_timestamp)
}

#[tauri::command]
pub fn live_import_is_ready() -> Result<bool, String> {
    imp::is_ready()
}

#[tauri::command]
pub fn live_import_stats() -> Result<SendStats, String> {
    let s = SEND_STATS.lock().unwrap();
    Ok(SendStats {
        total: s.total,
        ok: s.ok,
        err: s.err,
        last_hr: s.last_hr,
        last_err_msg: s.last_err_msg.clone(),
    })
}

#[tauri::command]
pub fn live_import_close() -> Result<(), String> {
    imp::close();
    Ok(())
}

#[tauri::command]
pub fn live_import_find_exe(wps_path: String) -> Result<Option<String>, String> {
    Ok(find_wps_exe(&wps_path))
}

#[tauri::command]
pub fn live_import_is_fts_running() -> Result<bool, String> {
    Ok(is_fts_running())
}

#[tauri::command]
pub fn launch_wps(wps_path: String) -> Result<(), String> {
    let exe = find_wps_exe(&wps_path).ok_or_else(|| format!("在 {} 中未找到 WPS 可执行文件", wps_path))?;
    std::process::Command::new(&exe).current_dir(&wps_path).arg("/ComProbe Protocol Analysis System=Generic").arg("/oemkey=Virtual").spawn()
        .map_err(|e| format!("启动 WPS 失败: {}", e))?;
    Ok(())
}
