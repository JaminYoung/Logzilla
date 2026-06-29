use tauri::State;
use std::sync::Arc;
use crate::AppState;
use crate::core::flash::engine::FlashEngine;
use crate::core::flash::progress::FlashProgress;
use log::info;

#[tauri::command]
pub async fn start_flash(
    dcf_data: Vec<u8>,
    port: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting flash on port {}", port);
    
    // Check if already flashing
    {
        let flash_engine = state.flash_engine.lock().map_err(|e| e.to_string())?;
        if flash_engine.is_running() {
            return Err("Flash already in progress".to_string());
        }
    }
    
    // Get the existing serial port handle
    let serial_port = {
        let ports = state.serial_ports.lock().map_err(|e| e.to_string())?;
        ports.get(&port)
            .ok_or_else(|| format!("Port {} is not connected", port))?
            .clone()
    };
    
    // Get flash engine and start flash
    let flash_engine = state.flash_engine.lock().map_err(|e| e.to_string())?;
    
    flash_engine.start_flash(dcf_data, serial_port)
        .map_err(|e| format!("Failed to start flash: {}", e))?;
    
    info!("Flash started successfully");
    Ok(())
}

#[tauri::command]
pub async fn cancel_flash(state: State<'_, AppState>) -> Result<(), String> {
    info!("Cancelling flash");
    
    let flash_engine = state.flash_engine.lock().map_err(|e| e.to_string())?;
    flash_engine.cancel();
    
    Ok(())
}

#[tauri::command]
pub async fn get_flash_progress(state: State<'_, AppState>) -> Result<FlashProgress, String> {
    let flash_engine = state.flash_engine.lock().map_err(|e| e.to_string())?;
    Ok(flash_engine.get_progress())
}
