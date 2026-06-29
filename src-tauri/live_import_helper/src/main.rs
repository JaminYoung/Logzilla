#![windows_subsystem = "windows"]

use libloading::Library;
use std::io::{self, BufRead, Write};

type HRESULT = i32;
const S_OK: HRESULT = 0;

#[link(name = "kernel32")]
extern "system" {
    fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn respond_ok() {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "OK");
    let _ = out.flush();
}

fn respond_ok_flag(flag: bool) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "OK|{}", if flag { 1 } else { 0 });
    let _ = out.flush();
}

fn respond_err(msg: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "ERR|{}", msg);
    let _ = out.flush();
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    s.split_whitespace()
        .map(|b| u8::from_str_radix(b, 16).map_err(|_| format!("无效十六进制: {}", b)))
        .collect()
}

fn main() {
    let wps_root = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            respond_err("缺少 WPS 路径参数");
            return;
        }
    };

    let dll_path = std::path::Path::new(&wps_root)
        .join("Executables")
        .join("Core")
        .join("LiveImportAPI.dll");

    let dll_path_str = dll_path.to_string_lossy().to_string();

    // Add Core directory to DLL search path for dependency resolution
    let core_dir = std::path::Path::new(&wps_root)
        .join("Executables")
        .join("Core");
    let core_dir_wide = to_wide(&core_dir.to_string_lossy());
    unsafe { SetDllDirectoryW(core_dir_wide.as_ptr()); }

    let _lib = match unsafe { Library::new(&dll_path_str) } {
        Ok(l) => {
            unsafe { SetDllDirectoryW(std::ptr::null()); }
            l
        },
        Err(e) => {
            respond_err(&format!("DLL 加载失败: {}", e));
            return;
        }
    };

    // Load all function pointers upfront
    macro_rules! load_fn {
        ($name:literal, $ty:ty) => {{
            let name_bytes: &[u8] = $name;
            unsafe {
                let sym: libloading::Symbol<unsafe extern "C" fn()> =
                    _lib.get::<unsafe extern "C" fn()>(name_bytes)
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
    let send_notification: unsafe extern "C" fn(i32) -> HRESULT =
        load_fn!(b"SendNotification", unsafe extern "C" fn(i32) -> HRESULT);

    let mut initialized = false;
    let mut frame_count: u64 = 0;
    let mut ok_count: u64 = 0;
    let mut err_count: u64 = 0;

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, '|').collect();
        let cmd = parts[0];

        match cmd {
            "INIT" => {
                if initialized {
                    respond_err("已初始化");
                    continue;
                }
                let args = parts.get(1).unwrap_or(&"").to_string();
                let args = args.replace('\x1E', "\n");
                let arg_parts: Vec<&str> = args.splitn(2, '|').collect();
                let conn_str = arg_parts.first().unwrap_or(&"");
                let config_str = arg_parts.get(1).unwrap_or(&"");

                let conn_cstr = std::ffi::CString::new(conn_str.as_bytes()).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                let config_cstr = std::ffi::CString::new(config_str.as_bytes()).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
                let mut success: i32 = 0;
                let hr = unsafe {
                    initialize(conn_cstr.as_ptr(), config_cstr.as_ptr(), &mut success)
                };

                if hr == S_OK && success != 0 {
                    initialized = true;
                    respond_ok();
                } else {
                    respond_err(&format!("初始化失败: HRESULT=0x{:x}, success={}", hr, success));
                }
            }

            "READY" => {
                if !initialized {
                    respond_err("未初始化");
                    continue;
                }
                let mut ready: i32 = 0;
                let hr = unsafe { is_app_ready(&mut ready) };
                if hr == S_OK {
                    respond_ok_flag(ready != 0);
                } else {
                    respond_err(&format!("IsAppReady 失败: 0x{:x}", hr));
                }
            }

            "NOTIFY" => {
                if !initialized {
                    respond_err("未初始化");
                    continue;
                }
                let evt = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(19);
                let hr = unsafe { send_notification(evt) };
                if hr == S_OK {
                    respond_ok();
                } else {
                    respond_err(&format!("SendNotification 失败: 0x{:x}", hr));
                }
            }

            "FRAME" => {
                frame_count += 1;
                if !initialized {
                    eprintln!("[HCI #{frame_count}] SKIP: 未初始化");
                    respond_err("未初始化");
                    continue;
                }
                let args = parts.get(1).unwrap_or(&"");
                let arg_parts: Vec<&str> = args.splitn(4, '|').collect();
                if arg_parts.len() < 4 {
                    eprintln!("[HCI #{frame_count}] SKIP: 参数不足");
                    respond_err("参数不足");
                    continue;
                }
                let drf: i32 = arg_parts[0].parse().unwrap_or(0);
                let side: i32 = arg_parts[1].parse().unwrap_or(0);
                let ts: i64 = arg_parts[2].parse().unwrap_or(0);
                let data = match parse_hex(arg_parts[3]) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[HCI #{frame_count}] SKIP: {e}");
                        respond_err(&e);
                        continue;
                    }
                };

                let len = data.len() as i32;
                eprintln!("[HCI #{frame_count}] drf={drf} side={side} len={len} ts={ts}");
                let hr = unsafe {
                    send_frame(len, len, data.as_ptr(), drf, side, ts)
                };
                if hr == S_OK {
                    ok_count += 1;
                    eprintln!("[HCI #{frame_count}] OK");
                    respond_ok();
                } else {
                    err_count += 1;
                    eprintln!("[HCI #{frame_count}] FAIL hr=0x{hr:x}");
                    respond_err(&format!("SendFrame 失败: 0x{:x}", hr));
                }
            }

            "CLOSE" => {
                if initialized {
                    unsafe { release(); }
                    initialized = false;
                }
                respond_ok();
                break;
            }

            _ => {
                respond_err(&format!("未知命令: {}", cmd));
            }
        }

    }
}
