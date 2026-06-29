use anyhow::Result;
use crate::core::serial::port::SerialPortHandle;

pub struct FlashProtocol {
    port: std::sync::Arc<SerialPortHandle>,
}

impl FlashProtocol {
    pub fn new(port: std::sync::Arc<SerialPortHandle>) -> Self {
        Self { port }
    }

    pub fn enter_download_mode(&self) -> Result<()> {
        self.port.send(&[0x02, 0x00, 0x00, 0x00])?;
        Ok(())
    }

    pub fn erase_flash(&self) -> Result<()> {
        self.port.send(b"ERASE")?;
        Ok(())
    }

    pub fn write_flash(&self, offset: u32, data: &[u8]) -> Result<()> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&offset.to_le_bytes());
        packet.extend_from_slice(&(data.len() as u32).to_le_bytes());
        packet.extend_from_slice(data);
        self.port.send(&packet)?;
        Ok(())
    }

    pub fn verify_flash(&self, offset: u32, data: &[u8]) -> Result<()> {
        let cmd = format!("VERIFY:{}:{}", offset, data.len());
        self.port.send(cmd.as_bytes())?;
        Ok(())
    }

    pub fn reset_device(&self) -> Result<()> {
        self.port.send(b"RESET")?;
        Ok(())
    }
}
