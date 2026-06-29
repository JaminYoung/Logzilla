use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::port::SerialPortHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub description: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
    let ports = serialport::available_ports()
        .map_err(|e| anyhow!("Failed to list serial ports: {}", e))?;

    let mut result = Vec::new();
    for port in ports {
        let vid = match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => Some(info.vid),
            _ => None,
        };
        let pid = match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => Some(info.pid),
            _ => None,
        };

        result.push(SerialPortInfo {
            port_name: port.port_name.clone(),
            description: format!("{}", port.port_name),
            vid,
            pid,
        });
    }

    Ok(result)
}

pub struct SerialManager {
    ports: Arc<Mutex<HashMap<String, Arc<SerialPortHandle>>>>,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            ports: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn open_port(&self, port_name: &str, baudrate: u32) -> Result<()> {
        let mut ports = self.ports.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        if ports.contains_key(port_name) {
            return Err(anyhow!("Port {} is already open", port_name));
        }

        let handle = SerialPortHandle::open(port_name, baudrate)?;
        ports.insert(port_name.to_string(), Arc::new(handle));

        Ok(())
    }

    pub fn close_port(&self, port_name: &str) -> Result<()> {
        let mut ports = self.ports.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        ports.remove(port_name);
        Ok(())
    }

    pub fn send(&self, port_name: &str, data: &[u8]) -> Result<()> {
        let ports = self.ports.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let handle = ports.get(port_name)
            .ok_or_else(|| anyhow!("Port {} not found", port_name))?;
        handle.send(data)
    }

    pub fn is_open(&self, port_name: &str) -> bool {
        let ports = self.ports.lock().unwrap_or_else(|e| e.into_inner());
        ports.contains_key(port_name)
    }

    pub fn active_ports(&self) -> Vec<String> {
        let ports = self.ports.lock().unwrap_or_else(|e| e.into_inner());
        ports.keys().cloned().collect()
    }

    pub fn get_port_handle(&self, port_name: &str) -> Option<Arc<SerialPortHandle>> {
        let ports = self.ports.lock().unwrap_or_else(|e| e.into_inner());
        ports.get(port_name).cloned()
    }
}

impl Default for SerialManager {
    fn default() -> Self {
        Self::new()
    }
}
