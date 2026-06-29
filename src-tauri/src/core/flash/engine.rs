use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use std::thread;
use super::progress::{FlashProgress, FlashState};
use super::protocol::FlashProtocol;
use crate::core::serial::port::SerialPortHandle;

pub struct FlashEngine {
    progress: Arc<Mutex<FlashProgress>>,
    running: Arc<Mutex<bool>>,
}

impl FlashEngine {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(FlashProgress::default())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_flash(&self, dcf_data: Vec<u8>, port: Arc<SerialPortHandle>) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        if *running {
            return Err(anyhow!("Flash already in progress"));
        }
        *running = true;
        drop(running);

        let progress = self.progress.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            if let Err(e) = flash_worker(dcf_data, port, progress.clone(), running.clone()) {
                let mut p = progress.lock().unwrap();
                p.set_state(FlashState::Error(e.to_string()), 0, &format!("Error: {}", e));
                let mut r = running.lock().unwrap();
                *r = false;
            }
        });

        Ok(())
    }

    pub fn cancel(&self) {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        *running = false;
    }

    pub fn get_progress(&self) -> FlashProgress {
        self.progress.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn flash_worker(
    dcf_data: Vec<u8>,
    port: Arc<SerialPortHandle>,
    progress: Arc<Mutex<FlashProgress>>,
    running: Arc<Mutex<bool>>,
) -> Result<()> {
    let proto = FlashProtocol::new(port);
    let total_size = dcf_data.len();
    let chunk_size = 4096;

    // Update progress
    {
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Connecting, 0, "正在连接芯片...");
    }

    // Enter download mode
    proto.enter_download_mode()?;
    thread::sleep(std::time::Duration::from_millis(100));

    // Check if cancelled
    if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
        return Ok(());
    }

    // Erase flash
    {
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Erasing, 5, "正在擦除Flash...");
    }
    thread::sleep(std::time::Duration::from_millis(500));

    proto.erase_flash()?;
    thread::sleep(std::time::Duration::from_millis(500));

    // Check if cancelled
    if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
        return Ok(());
    }

    // Write flash
    {
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Writing, 10, "正在写入固件...");
    }

    for offset in (0..total_size).step_by(chunk_size) {
        // Check if cancelled
        if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
            return Ok(());
        }

        let end = (offset + chunk_size).min(total_size);
        let chunk = &dcf_data[offset..end];
        proto.write_flash(offset as u32, chunk)?;

        let percent = 10 + ((offset as f64 / total_size as f64) * 80.0) as u8;
        let msg = format!("写入中... {}KB / {}KB", offset / 1024, total_size / 1024);
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Writing, percent, &msg);

        thread::sleep(std::time::Duration::from_millis(10));
    }

    // Check if cancelled
    if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
        return Ok(());
    }

    // Verify flash
    {
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Verifying, 90, "正在校验固件...");
    }

    for offset in (0..total_size).step_by(chunk_size) {
        // Check if cancelled
        if !*running.lock().unwrap_or_else(|e| e.into_inner()) {
            return Ok(());
        }

        let end = (offset + chunk_size).min(total_size);
        let chunk = &dcf_data[offset..end];
        proto.verify_flash(offset as u32, chunk)?;

        thread::sleep(std::time::Duration::from_millis(10));
    }

    // Done
    {
        let mut p = progress.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        p.set_state(FlashState::Done, 100, "烧录完成！");
    }

    // Reset device
    proto.reset_device()?;
    thread::sleep(std::time::Duration::from_millis(500));

    let mut r = running.lock().unwrap_or_else(|e| e.into_inner());
    *r = false;

    Ok(())
}
