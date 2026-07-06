use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[allow(dead_code)]
type Hresult = i32;
#[allow(dead_code)]
const S_OK: Hresult = 0;

#[allow(dead_code)]
fn pc_timestamp_to_filetime(ts: &str) -> i64 {
    let ts = ts.trim();
    if ts.len() < 12 {
        return current_filetime();
    }

    let h: u32 = match ts[0..2].parse() {
        Ok(v) => v,
        Err(_) => return current_filetime(),
    };
    let m: u32 = match ts[3..5].parse() {
        Ok(v) => v,
        Err(_) => return current_filetime(),
    };
    let s: u32 = match ts[6..8].parse() {
        Ok(v) => v,
        Err(_) => return current_filetime(),
    };
    let ms: u32 = match ts[9..12].parse() {
        Ok(v) => v,
        Err(_) => return current_filetime(),
    };

    let Some(time) = chrono::NaiveTime::from_hms_milli_opt(h, m, s, ms) else {
        return current_filetime();
    };
    let local_naive = chrono::NaiveDateTime::new(chrono::Local::now().date_naive(), time);
    let Some(local_dt) = local_naive.and_local_timezone(chrono::Local).single() else {
        return current_filetime();
    };

    unix_usec_to_filetime(local_dt.with_timezone(&chrono::Utc).timestamp_micros())
}

#[allow(dead_code)]
fn current_filetime() -> i64 {
    unix_usec_to_filetime(chrono::Utc::now().timestamp_micros())
}

#[allow(dead_code)]
fn unix_usec_to_filetime(unix_usec: i64) -> i64 {
    let unix_sec = unix_usec / 1_000_000;
    let sub_us = unix_usec % 1_000_000;
    (unix_sec + 11_644_473_600) * 10_000_000 + sub_us * 10
}

fn parse_ini(path: &Path) -> Result<(String, String), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read INI {}: {}", path.display(), e))?;
    let mut connection_string = String::new();
    let mut config_lines: Vec<String> = Vec::new();
    let mut in_config = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_config = trimmed.eq_ignore_ascii_case("[configuration]");
            continue;
        }
        if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("ConnectionString=") {
            connection_string = value.to_string();
        } else if in_config {
            config_lines.push(trimmed.to_string());
        }
    }

    if connection_string.is_empty() {
        return Err(format!(
            "ConnectionString not found in INI {}",
            path.display()
        ));
    }

    Ok((connection_string, config_lines.join("\n") + "\n"))
}

fn find_named_file_recursive(root: &Path, file_name: &str) -> Option<PathBuf> {
    if root.is_file() {
        return root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| name.eq_ignore_ascii_case(file_name))
            .map(|_| root.to_path_buf());
    }

    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_file() {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.eq_ignore_ascii_case(file_name))
                .unwrap_or(false)
            {
                return Some(path);
            }
        } else if path.is_dir() {
            if let Some(found) = find_named_file_recursive(&path, file_name) {
                return Some(found);
            }
        }
    }

    None
}

fn find_preferred_file(wps_path: &str, preferred_relative: &[&str], file_name: &str) -> Option<PathBuf> {
    let root = Path::new(wps_path);
    if !preferred_relative.is_empty() {
        let preferred = preferred_relative
            .iter()
            .fold(root.to_path_buf(), |acc, part| acc.join(part));
        if preferred.exists() {
            return Some(preferred);
        }
    }
    find_named_file_recursive(root, file_name)
}

fn find_liveimport_ini(wps_path: &str) -> Option<PathBuf> {
    find_preferred_file(wps_path, &["liveimport.ini"], "liveimport.ini")
}

fn find_liveimport_dll(wps_path: &str) -> Option<PathBuf> {
    #[cfg(target_pointer_width = "32")]
    {
        find_preferred_file(
            wps_path,
            &["Executables", "Core", "LiveImportAPI.dll"],
            "LiveImportAPI.dll",
        )
    }

    #[cfg(target_pointer_width = "64")]
    {
        find_preferred_file(
            wps_path,
            &["Executables", "Core", "LiveImportAPI_x64.dll"],
            "LiveImportAPI_x64.dll",
        )
    }
}

fn find_wps_exe(wps_path: &str) -> Option<PathBuf> {
    find_preferred_file(wps_path, &["Executables", "Core", "Fts.exe"], "Fts.exe")
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

static SEND_STATS: Mutex<SendStats> = Mutex::new(SendStats {
    total: 0,
    ok: 0,
    err: 0,
    last_hr: 0,
    last_err_msg: String::new(),
});

#[allow(dead_code)]
fn inc_send_ok() {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.ok += 1;
    s.last_hr = 0;
}

#[allow(dead_code)]
fn inc_send_err(hr: i32, msg: String) {
    let mut s = SEND_STATS.lock().unwrap();
    s.total += 1;
    s.err += 1;
    s.last_hr = hr;
    s.last_err_msg = msg.clone();
    eprintln!("[LOGZILLA] SendFrame ERR: 0x{:x} {}", hr, msg);
}

#[cfg(target_pointer_width = "64")]
mod imp {
    use super::*;

    pub fn init(wps_path: &str) -> Result<LiveImportDiag, String> {
        let conn_string = find_liveimport_ini(wps_path)
            .and_then(|ini_path| parse_ini(&ini_path).ok())
            .map(|(c, _)| c)
            .unwrap_or_default();

        Ok(LiveImportDiag {
            conn_string,
            dll_loaded: find_liveimport_dll(wps_path).is_some(),
            init_hr: -1,
            init_success: 0,
            fts_running: is_fts_running(),
        })
    }

    pub fn send_frame(
        _data_hex: &str,
        _h4_type: u8,
        _sent: bool,
        _pc_timestamp: &str,
    ) -> Result<(), String> {
        Err("Live Import is only available in the 32-bit build".to_string())
    }

    pub fn is_ready() -> Result<bool, String> {
        Ok(false)
    }

    pub fn close() {
        *SEND_STATS.lock().unwrap() = SendStats::default();
    }
}

#[cfg(target_pointer_width = "32")]
mod imp {
    use super::*;
    use libloading::Library;
    use std::ffi::CString;

    #[link(name = "kernel32")]
    extern "system" {
        fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
    }

    struct DirectDll {
        _lib: Library,
        release: unsafe extern "C" fn() -> Hresult,
        is_app_ready: unsafe extern "C" fn(*mut bool) -> Hresult,
        send_frame: unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> Hresult,
    }
    unsafe impl Send for DirectDll {}
    unsafe impl Sync for DirectDll {}

    static LIVE_IMPORT: Mutex<Option<DirectDll>> = Mutex::new(None);

    pub fn init(wps_path: &str) -> Result<LiveImportDiag, String> {
        let ini_path = find_liveimport_ini(wps_path)
            .ok_or_else(|| format!("liveimport.ini not found under {}", wps_path))?;
        let (conn_str, config_str) = parse_ini(&ini_path)?;
        let fts_running = is_fts_running();

        let Some(dll_path) = find_liveimport_dll(wps_path) else {
            return Ok(LiveImportDiag {
                conn_string: conn_str,
                dll_loaded: false,
                init_hr: -1,
                init_success: 0,
                fts_running,
            });
        };

        let dll_dir = dll_path.parent().unwrap_or_else(|| Path::new(wps_path));
        let dll_dir_wide = to_utf16(&dll_dir.to_string_lossy());
        unsafe {
            SetDllDirectoryW(dll_dir_wide.as_ptr());
        }

        let lib = match unsafe { Library::new(&dll_path) } {
            Ok(l) => {
                unsafe {
                    SetDllDirectoryW(std::ptr::null());
                }
                l
            }
            Err(e) => {
                unsafe {
                    SetDllDirectoryW(std::ptr::null());
                }
                eprintln!("[LOGZILLA] failed to load {}: {}", dll_path.display(), e);
                return Ok(LiveImportDiag {
                    conn_string: conn_str,
                    dll_loaded: false,
                    init_hr: -1,
                    init_success: 0,
                    fts_running,
                });
            }
        };

        macro_rules! load_fn {
            ($name:literal, $ty:ty) => {{
                let sym = unsafe { lib.get::<$ty>($name) }
                    .map_err(|e| format!("function load failed {:?}: {}", $name, e))?;
                *sym
            }};
        }

        let initialize: unsafe extern "C" fn(
            *const std::ffi::c_char,
            *const std::ffi::c_char,
            *mut bool,
        ) -> Hresult = load_fn!(
            b"InitializeLiveImport",
            unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char, *mut bool) -> Hresult
        );
        let release: unsafe extern "C" fn() -> Hresult =
            load_fn!(b"ReleaseLiveImport", unsafe extern "C" fn() -> Hresult);
        let is_app_ready: unsafe extern "C" fn(*mut bool) -> Hresult =
            load_fn!(b"IsAppReady", unsafe extern "C" fn(*mut bool) -> Hresult);
        let send_frame: unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> Hresult =
            load_fn!(
                b"SendFrame",
                unsafe extern "C" fn(i32, i32, *const u8, i32, i32, i64) -> Hresult
            );

        let conn_cstr =
            CString::new(conn_str.as_bytes()).map_err(|_| "invalid connection string".to_string())?;
        let config_cstr =
            CString::new(config_str.as_bytes()).map_err(|_| "invalid config string".to_string())?;

        let mut success = false;
        let hr = unsafe { initialize(conn_cstr.as_ptr(), config_cstr.as_ptr(), &mut success) };

        let dll = DirectDll {
            _lib: lib,
            release,
            is_app_ready,
            send_frame,
        };

        if hr == S_OK && success {
            *LIVE_IMPORT.lock().unwrap() = Some(dll);
        }

        Ok(LiveImportDiag {
            conn_string: conn_str,
            dll_loaded: true,
            init_hr: hr,
            init_success: i32::from(success),
            fts_running,
        })
    }

    pub fn send_frame(
        data_hex: &str,
        h4_type: u8,
        sent: bool,
        pc_timestamp: &str,
    ) -> Result<(), String> {
        let g = LIVE_IMPORT.lock().unwrap();
        let dll = g.as_ref().ok_or("Live Import not initialized")?;
        let data: Vec<u8> = data_hex
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).map_err(|_| format!("invalid hex byte: {}", b)))
            .collect::<Result<Vec<u8>, String>>()?;
        if data.is_empty() {
            return Err("empty frame".to_string());
        }

        let drf: i32 = 1 << ((h4_type as i32) - 1);
        let side: i32 = if sent { 0 } else { 1 };
        let ts = if pc_timestamp.is_empty() {
            current_filetime()
        } else {
            pc_timestamp_to_filetime(pc_timestamp)
        };
        let len = data.len() as i32;
        let hr = unsafe { (dll.send_frame)(len, len, data.as_ptr(), drf, side, ts) };
        eprintln!("[LOGZILLA] send_frame drf={drf} side={side} len={len} hr=0x{hr:x}");

        if hr == S_OK {
            inc_send_ok();
            Ok(())
        } else {
            let msg = format!("SendFrame failed: 0x{:x}", hr);
            inc_send_err(hr, msg.clone());
            Err(msg)
        }
    }

    pub fn is_ready() -> Result<bool, String> {
        let g = LIVE_IMPORT.lock().unwrap();
        let dll = g.as_ref().ok_or("Live Import not initialized")?;
        let mut ready = false;
        let hr = unsafe { (dll.is_app_ready)(&mut ready) };
        if hr == S_OK {
            Ok(ready)
        } else {
            Err(format!("IsAppReady failed: 0x{:x}", hr))
        }
    }

    pub fn close() {
        let mut g = LIVE_IMPORT.lock().unwrap();
        if let Some(dll) = g.take() {
            let hr = unsafe { (dll.release)() };
            eprintln!("[LOGZILLA] live_import_close: ReleaseLiveImport hr=0x{:x}", hr);
        }
        *SEND_STATS.lock().unwrap() = SendStats::default();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[tauri::command]
pub fn live_import_init(wps_path: String) -> Result<LiveImportDiag, String> {
    imp::init(&wps_path)
}

#[tauri::command]
pub fn live_import_send_frame(
    data_hex: String,
    h4_type: u8,
    sent: bool,
    pc_timestamp: String,
) -> Result<(), String> {
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
    let exe = find_wps_exe(&wps_path)
        .ok_or_else(|| format!("Fts.exe not found under {}", wps_path))?;
    let current_dir = exe.parent().unwrap_or_else(|| Path::new(&wps_path)).to_path_buf();
    std::process::Command::new(&exe)
        .current_dir(current_dir)
        .arg("/ComProbe Protocol Analysis System=Generic")
        .arg("/oemkey=Virtual")
        .spawn()
        .map_err(|e| format!("failed to launch WPS: {}", e))?;
    Ok(())
}
