use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use crate::core::serial::port::SerialPortHandle;
use chrono::Local;

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

    let bt_value_names: Vec<String> = key.enum_values()
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
    use winreg::enums::KEY_READ;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    use std::path::Path;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let base_path = r"SYSTEM\CurrentControlSet\Enum\BTHENUM";
    let base_key = match hklm.open_subkey_with_flags(base_path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return None,
    };

    for dev_result in base_key.enum_keys() {
        let dev = match dev_result { Ok(d) => d, _ => continue };
        let dev_key = match base_key.open_subkey_with_flags(&dev, KEY_READ) {
            Ok(k) => k, _ => continue
        };
        for inst_result in dev_key.enum_keys() {
            let inst = match inst_result { Ok(i) => i, _ => continue };
            let inst_path = Path::new(base_path).join(&dev).join(&inst);
            let inst_key = match hklm.open_subkey_with_flags(
                inst_path.to_str().unwrap(), KEY_READ
            ) {
                Ok(k) => k, _ => continue
            };
            if let Ok(port_cfg) = inst_key.open_subkey_with_flags(r"Device Parameters\Port", KEY_READ) {
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
    let ports = serialport::available_ports()
        .map_err(|e| format!("Failed to list serial ports: {}", e))?;

    let bt_ports = get_bluetooth_ports();
    let mut result = Vec::new();
    for port in ports {
        let (vid, pid, mut port_type, mut desc_fallback) = match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => (
                Some(info.vid),
                Some(info.pid),
                "usb",
                info.product.clone().or_else(|| info.manufacturer.clone()).unwrap_or_default(),
            ),
            serialport::SerialPortType::BluetoothPort => {
                (None, None, "bluetooth", "Bluetooth SPP".to_string())
            },
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
pub fn close_serial_port(
    port: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
    ports.remove(&port);

    let mut connected = state.connected_ports.lock().map_err(|e| e.to_string())?;
    connected.retain(|p| p != &port);

    Ok(())
}

#[tauri::command]
pub fn send_serial_data(
    port: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
    let serial_port = ports.get(&port)
        .ok_or_else(|| format!("Port {} not found", port))?;

    serial_port.send(&data)
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
    let serial_port = ports.get(&port)
        .ok_or_else(|| format!("Port {} not found", port))?;

    let mut buf = vec![0u8; 4096];
    let bytes_read = serial_port.read(&mut buf)
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
        let new_lines: Vec<String> = parts.iter()
            .map(|l| l.trim_end_matches('\r'))  // 去掉 \r
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
                (state.logs2.lock().map_err(|e| e.to_string())?,
                 state.logs2_version.lock().map_err(|e| e.to_string())?)
            } else {
                (state.logs.lock().map_err(|e| e.to_string())?,
                 state.logs_version.lock().map_err(|e| e.to_string())?)
            };

            logs.extend(new_lines);

            // 限制最大行数：从头丢弃旧行，并把丢弃数累加到 base 游标，
            // 使前端的累计行号仍能正确映射到缓冲区内下标。
            if logs.len() > max_lines {
                let drain_count = logs.len() - max_lines;
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
            let mut buffers = state.lc3_capture_buffers.lock().map_err(|e| e.to_string())?;
            if let Some(buffer) = buffers.get_mut(&port) {
                buffer.extend_from_slice(&buf);
            }
        }
    }

    Ok(buf)
}
