use std::sync::Mutex;

// ========= Shared utilities (always compiled) =========

#[allow(dead_code)]
type HRESULT = i32;
#[allow(dead_code)]
const S_OK: HRESULT = 0;

#[allow(dead_code)] // 仅 32 位直连路径使用
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

#[allow(dead_code)] // 仅 32 位直连路径使用
fn current_filetime() -> i64 {
    unix_usec_to_filetime(chrono::Utc::now().timestamp_micros())
}

#[allow(dead_code)] // 仅 32 位直连路径使用
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

#[allow(dead_code)] // 仅 32 位直连路径使用
fn inc_send_ok() {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.ok += 1;
    s.last_hr = 0;
}

#[allow(dead_code)] // 仅 32 位直连路径使用
fn inc_send_err(hr: i32, msg: String) {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.err += 1;
    s.last_hr = hr;
    s.last_err_msg = msg.clone();
    eprintln!("[LOGZILLA] SendFrame ERR: 0x{:x} {}", hr, msg);
}

// ========= 64-bit: Live Import 不可用（发布走 32 位直连 DLL 方案） =========
// 64 位进程无法加载 32 位厂商 DLL（LiveImportAPI.dll）。原先靠一个 32 位 helper
// 子进程桥接，已随 32 位直连方案一并移除。64 位（dev）构建下 Live Import 返回
// 诊断而非 panic；正式功能在 32 位构建（见下方 32-bit 实现）。
#[cfg(target_pointer_width = "64")]
mod imp {
    use super::*;

    pub fn init(wps_path: &str) -> Result<LiveImportDiag, String> {
        let ini_path = std::path::Path::new(wps_path).join("liveimport.ini");
        let conn_string = parse_ini(ini_path.to_str().unwrap_or(""))
            .map(|(c, _)| c)
            .unwrap_or_default();
        Ok(LiveImportDiag {
            conn_string,
            dll_loaded: false,
            init_hr: -1,
            init_success: 0,
            fts_running: is_fts_running(),
        })
    }

    pub fn send_frame(_data_hex: &str, _h4_type: u8, _sent: bool, _pc_timestamp: &str) -> Result<(), String> {
        Err("Live Import 仅在 32 位构建可用（当前为 64 位 dev 构建）".to_string())
    }

    pub fn is_ready() -> Result<bool, String> {
        Ok(false)
    }

    pub fn close() {
        *SEND_STATS.lock().unwrap() = SendStats::default();
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
