use anyhow::{Result, anyhow};
use serialport::SerialPort;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SerialPortHandle {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    port_name: String,
    baudrate: u32,
}

impl SerialPortHandle {
    pub fn open(port_name: &str, baudrate: u32) -> Result<Self> {
        let port = serialport::new(port_name, baudrate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_millis(50))
            .open()
            .map_err(|e| anyhow!("Failed to open serial port {}: {}", port_name, e))?;

        Ok(Self {
            port: Arc::new(Mutex::new(port)),
            port_name: port_name.to_string(),
            baudrate,
        })
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        let mut port = self.port.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        port.write_all(data)
            .map_err(|e| anyhow!("Send error: {}", e))?;
        Ok(())
    }

    /// Reads data, returning the raw `io::Result` so the caller can inspect
    /// `ErrorKind` / `raw_os_error()` to distinguish a normal read timeout
    /// (`ErrorKind::TimedOut`) from a fatal device-removed error (raw OS
    /// errors 2/3/5 on Windows), enabling event-driven disconnect detection.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut port = self.port.lock().map_err(|e| io::Error::other(e.to_string()))?;
        port.read(buf)
    }

    pub fn available(&self) -> Result<usize> {
        let port = self.port.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let n = port.bytes_to_read()
            .map_err(|e| anyhow!("Available error: {}", e))?;
        Ok(n as usize)
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    pub fn baudrate(&self) -> u32 {
        self.baudrate
    }
}
