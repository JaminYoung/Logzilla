use crate::core::serial::port::SerialPortHandle;
use crate::{AppState, SerialReaderConfig, SerialReaderControl};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub description: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub port_type: String,
}

#[cfg(target_os = "windows")]
fn get_bluetooth_ports() -> Vec<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey(r"HARDWARE\DEVICEMAP\SERIALCOMM") {
        Ok(k) => k,
        Err(_) => return Vec::new(),
    };

    let bt_value_names: Vec<String> = key
        .enum_values()
        .filter_map(|r| r.ok())
        .filter(|(name, _)| name.to_uppercase().contains("BTH"))
        .map(|(name, _)| name)
        .collect();

    let mut bt_ports = Vec::new();
    for name in &bt_value_names {
        if let Ok(port) = key.get_value::<String, _>(name) {
            bt_ports.push(port);
        }
    }
    bt_ports
}

#[cfg(target_os = "windows")]
fn get_bluetooth_description(port_name: &str) -> Option<String> {
    use std::path::Path;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::enums::KEY_READ;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let base_path = r"SYSTEM\CurrentControlSet\Enum\BTHENUM";
    let base_key = match hklm.open_subkey_with_flags(base_path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return None,
    };

    for dev_result in base_key.enum_keys() {
        let dev = match dev_result {
            Ok(d) => d,
            _ => continue,
        };
        let dev_key = match base_key.open_subkey_with_flags(&dev, KEY_READ) {
            Ok(k) => k,
            _ => continue,
        };
        for inst_result in dev_key.enum_keys() {
            let inst = match inst_result {
                Ok(i) => i,
                _ => continue,
            };
            let inst_path = Path::new(base_path).join(&dev).join(&inst);
            let inst_key = match hklm.open_subkey_with_flags(inst_path.to_str().unwrap(), KEY_READ)
            {
                Ok(k) => k,
                _ => continue,
            };
            if let Ok(port_cfg) =
                inst_key.open_subkey_with_flags(r"Device Parameters\Port", KEY_READ)
            {
                if let Ok(com) = port_cfg.get_value::<String, _>("COM") {
                    if com == port_name {
                        if let Ok(name) = inst_key.get_value::<String, _>("FriendlyName") {
                            return Some(name);
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn get_bluetooth_ports() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
fn get_bluetooth_description(_port_name: &str) -> Option<String> {
    None
}

#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    let ports =
        serialport::available_ports().map_err(|e| format!("Failed to list serial ports: {}", e))?;

    let bt_ports = get_bluetooth_ports();
    let mut result = Vec::new();
    for port in ports {
        let (vid, pid, mut port_type, mut desc_fallback) = match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => (
                Some(info.vid),
                Some(info.pid),
                "usb",
                info.product
                    .clone()
                    .or_else(|| info.manufacturer.clone())
                    .unwrap_or_default(),
            ),
            serialport::SerialPortType::BluetoothPort => {
                (None, None, "bluetooth", "Bluetooth SPP".to_string())
            }
            serialport::SerialPortType::PciPort => (None, None, "pci", String::new()),
            serialport::SerialPortType::Unknown => (None, None, "unknown", String::new()),
        };

        if bt_ports.contains(&port.port_name) {
            port_type = "bluetooth";
            desc_fallback = get_bluetooth_description(&port.port_name)
                .unwrap_or_else(|| "Bluetooth SPP".to_string());
        }

        result.push(SerialPortInfo {
            port_name: port.port_name.clone(),
            description: desc_fallback,
            vid,
            pid,
            port_type: port_type.to_string(),
        });
    }

    Ok(result)
}

fn append_log_lines(
    state: &AppState,
    panel: usize,
    max_lines: usize,
    new_lines: Vec<String>,
) -> Result<(), String> {
    if new_lines.is_empty() {
        return Ok(());
    }

    let (mut logs, mut version) = if panel == 1 {
        (
            state.logs2.lock().map_err(|e| e.to_string())?,
            state.logs2_version.lock().map_err(|e| e.to_string())?,
        )
    } else {
        (
            state.logs.lock().map_err(|e| e.to_string())?,
            state.logs_version.lock().map_err(|e| e.to_string())?,
        )
    };

    logs.extend(new_lines);

    let retention_lines = max_lines.max(1);

    if logs.len() > retention_lines {
        let drain_count = logs.len() - retention_lines;
        logs.drain(0..drain_count);
        let mut base = if panel == 1 {
            state.logs2_base.lock().map_err(|e| e.to_string())?
        } else {
            state.logs_base.lock().map_err(|e| e.to_string())?
        };
        *base += drain_count;
    }

    *version += 1;
    Ok(())
}

fn append_auto_save_lines(path: &str, lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Ok(());
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open auto-save file: {}", e))?;

    for line in lines {
        file.write_all(line.as_bytes())
            .map_err(|e| format!("Failed to write auto-save file: {}", e))?;
        file.write_all(b"\n")
            .map_err(|e| format!("Failed to write auto-save file: {}", e))?;
    }
    file.flush()
        .map_err(|e| format!("Failed to flush auto-save file: {}", e))?;
    Ok(())
}

fn append_lc3_capture(state: &AppState, port: &str, buf: &[u8]) -> Result<(), String> {
    let active = state.lc3_capture_active.lock().map_err(|e| e.to_string())?;
    if active.contains(&port.to_string()) {
        let mut buffers = state
            .lc3_capture_buffers
            .lock()
            .map_err(|e| e.to_string())?;
        if let Some(buffer) = buffers.get_mut(port) {
            buffer.extend_from_slice(buf);
        }
    }
    Ok(())
}

fn format_log_line(line: &[u8], timestamp: Option<&str>) -> Option<String> {
    let mut end = line.len();
    while end > 0 && line[end - 1] == b'\r' {
        end -= 1;
    }
    if end == 0 {
        return None;
    }

    let text = String::from_utf8_lossy(&line[..end]);
    Some(match timestamp {
        Some(ts) => format!("({}) {}", ts, text),
        None => text.to_string(),
    })
}

fn stop_serial_reader_inner(state: &AppState, port: &str) -> Result<(), String> {
    if let Some(reader) = state
        .serial_readers
        .lock()
        .map_err(|e| e.to_string())?
        .remove(port)
    {
        reader.stop.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn open_serial_port(
    port: String,
    baudrate: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut ports = state.serial_ports.lock().map_err(|e| e.to_string())?;

    if ports.contains_key(&port) {
        return Err(format!("Port {} is already open", port));
    }

    let serial_port = SerialPortHandle::open(&port, baudrate)
        .map_err(|e| format!("Failed to open port {}: {}", port, e))?;

    ports.insert(port.clone(), Arc::new(serial_port));

    let mut connected = state.connected_ports.lock().map_err(|e| e.to_string())?;
    connected.push(port);

    Ok(())
}

#[tauri::command]
pub fn close_serial_port(port: String, state: State<'_, AppState>) -> Result<(), String> {
    stop_serial_reader_inner(&state, &port)?;

    let mut ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
    ports.remove(&port);

    let mut connected = state.connected_ports.lock().map_err(|e| e.to_string())?;
    connected.retain(|p| p != &port);

    Ok(())
}

#[tauri::command]
pub fn start_serial_reader(
    port: String,
    timestamp_enabled: bool,
    max_lines: usize,
    panel: usize,
    auto_save_path: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let serial_port = {
        let ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
        ports
            .get(&port)
            .cloned()
            .ok_or_else(|| format!("Port {} not found", port))?
    };

    let reader_config = SerialReaderConfig {
        timestamp_enabled,
        max_lines,
        panel,
        auto_save_path,
    };

    let mut readers = state.serial_readers.lock().map_err(|e| e.to_string())?;
    if let Some(reader) = readers.get(&port) {
        *reader.config.lock().map_err(|e| e.to_string())? = reader_config;
        return Ok(());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let config = Arc::new(std::sync::Mutex::new(reader_config));
    readers.insert(
        port.clone(),
        SerialReaderControl {
            stop: stop.clone(),
            config: config.clone(),
        },
    );
    drop(readers);

    thread::Builder::new()
        .name(format!("serial-reader-{}", port))
        .spawn(move || {
            let mut buf = vec![0u8; 4096];
            let mut line_buf: Vec<u8> = Vec::with_capacity(256);
            let mut line_timestamp: Option<String> = None;

            while !stop.load(Ordering::Relaxed) {
                let bytes_read = match serial_port.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                };

                if bytes_read == 0 {
                    continue;
                }

                let state = app_handle.state::<AppState>();
                let _ = append_lc3_capture(&state, &port, &buf[..bytes_read]);

                let mut completed_lines: Vec<String> = Vec::new();
                for &byte in &buf[..bytes_read] {
                    if byte == b'\n' {
                        if let Some(line) = format_log_line(&line_buf, line_timestamp.as_deref()) {
                            completed_lines.push(line);
                        }
                        line_buf.clear();
                        line_timestamp = None;
                        continue;
                    }

                    if line_buf.is_empty() {
                        let cfg = config.lock().ok();
                        line_timestamp = cfg
                            .as_ref()
                            .filter(|c| c.timestamp_enabled)
                            .map(|_| Local::now().format("%H:%M:%S%.3f").to_string());
                    }
                    line_buf.push(byte);
                }

                if !completed_lines.is_empty() {
                    if let Ok(cfg) = config.lock() {
                        let panel = cfg.panel;
                        let max_lines = cfg.max_lines;
                        let auto_save_path = cfg.auto_save_path.clone();
                        drop(cfg);

                        let state = app_handle.state::<AppState>();
                        if let Some(path) = auto_save_path.as_deref() {
                            if let Err(e) = append_auto_save_lines(path, &completed_lines) {
                                eprintln!("[LOGZILLA] auto-save failed: {}", e);
                            }
                        }
                        let _ = append_log_lines(&state, panel, max_lines, completed_lines);
                    }
                }
            }
        })
        .map_err(|e| format!("Failed to start serial reader: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn stop_serial_reader(port: String, state: State<'_, AppState>) -> Result<(), String> {
    stop_serial_reader_inner(&state, &port)
}

#[tauri::command]
pub fn send_serial_data(
    port: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
    let serial_port = ports
        .get(&port)
        .ok_or_else(|| format!("Port {} not found", port))?;

    serial_port
        .send(&data)
        .map_err(|e| format!("Failed to send data: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn read_serial_data(
    port: String,
    timestamp_enabled: bool,
    max_lines: usize,
    panel: usize,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
    let serial_port = ports
        .get(&port)
        .ok_or_else(|| format!("Port {} not found", port))?;

    let available = serial_port
        .available()
        .map_err(|e| format!("Failed to query data: {}", e))?;
    if available == 0 {
        return Ok(Vec::new());
    }

    let mut buf = vec![0u8; 4096];
    let bytes_read = serial_port
        .read(&mut buf)
        .map_err(|e| format!("Failed to read data: {}", e))?;

    buf.truncate(bytes_read);

    // 存储到 AppState（带时间戳）
    if !buf.is_empty() {
        // 取出该串口上次遗留的不完整行
        let mut partial_map = state.partial_lines.lock().map_err(|e| e.to_string())?;
        let partial = partial_map.remove(&port).unwrap_or_default();

        // 拼接上次残留 + 本次读取
        let raw = format!("{}{}", partial, String::from_utf8_lossy(&buf));

        // 按 \n 拆分，最后一段可能不完整（无换行符），存回 partial
        let mut parts: Vec<&str> = raw.split('\n').collect();
        let tail = parts.pop().unwrap_or("");
        // tail 不含 \n，是不完整的行，保留到下次
        if !tail.is_empty() {
            partial_map.insert(port.clone(), tail.to_string());
        }
        drop(partial_map);

        // 处理完整行（parts 中每个元素都以 \r 或 \n 结尾，或者空行）
        let new_lines: Vec<String> = parts
            .iter()
            .map(|l| l.trim_end_matches('\r')) // 去掉 \r
            .filter(|l| !l.is_empty())
            .map(|l| {
                if timestamp_enabled {
                    let now = Local::now();
                    let ts = now.format("%H:%M:%S%.3f").to_string();
                    format!("({}) {}", ts, l)
                } else {
                    l.to_string()
                }
            })
            .collect();

        if !new_lines.is_empty() {
            // 根据 panel 写入对应缓冲区
            let (mut logs, mut version) = if panel == 1 {
                (
                    state.logs2.lock().map_err(|e| e.to_string())?,
                    state.logs2_version.lock().map_err(|e| e.to_string())?,
                )
            } else {
                (
                    state.logs.lock().map_err(|e| e.to_string())?,
                    state.logs_version.lock().map_err(|e| e.to_string())?,
                )
            };

            logs.extend(new_lines);

            // 限制最大行数：从头丢弃旧行，并把丢弃数累加到 base 游标，
            // 使前端的累计行号仍能正确映射到缓冲区内下标。
            let retention_lines = max_lines.max(1);

            if logs.len() > retention_lines {
                let drain_count = logs.len() - retention_lines;
                logs.drain(0..drain_count);
                let mut base = if panel == 1 {
                    state.logs2_base.lock().map_err(|e| e.to_string())?
                } else {
                    state.logs_base.lock().map_err(|e| e.to_string())?
                };
                *base += drain_count;
            }

            // 更新版本号
            *version += 1;
        }

        // LC3 捕获
        let active = state.lc3_capture_active.lock().map_err(|e| e.to_string())?;
        if active.contains(&port) {
            let mut buffers = state
                .lc3_capture_buffers
                .lock()
                .map_err(|e| e.to_string())?;
            if let Some(buffer) = buffers.get_mut(&port) {
                buffer.extend_from_slice(&buf);
            }
        }
    }

    Ok(buf)
}
