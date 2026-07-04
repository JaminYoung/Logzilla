use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::core::lc3::decoder::{Lc3Decoder, DecodeConfig};
use crate::core::lc3::{export, importer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lc3DecodeResult {
    pub success: bool,
    pub frame_count: usize,
    pub frame_samples: usize,
    pub duration_secs: f64,
    pub leftover_bytes: usize,
    pub saved_files: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lc3CaptureStatus {
    pub port: String,
    pub byte_count: usize,
    pub active: bool,
}

#[tauri::command]
pub fn lc3_get_connected_ports(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let connected = state.connected_ports.lock().map_err(|e| e.to_string())?;
    Ok(connected.clone())
}

#[tauri::command]
pub fn lc3_start_capture(
    port: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut buffers = state.lc3_capture_buffers.lock().map_err(|e| e.to_string())?;
    buffers.insert(port.clone(), Vec::new());

    let mut active = state.lc3_capture_active.lock().map_err(|e| e.to_string())?;
    if !active.contains(&port) {
        active.push(port);
    }

    Ok(())
}

#[tauri::command]
pub fn lc3_stop_capture(
    port: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let mut active = state.lc3_capture_active.lock().map_err(|e| e.to_string())?;
    active.retain(|p| p != &port);

    let mut buffers = state.lc3_capture_buffers.lock().map_err(|e| e.to_string())?;
    let data = buffers.remove(&port).unwrap_or_default();

    Ok(data)
}

#[tauri::command]
pub fn lc3_get_capture_status(
    port: String,
    state: State<'_, AppState>,
) -> Result<Lc3CaptureStatus, String> {
    let active = state.lc3_capture_active.lock().map_err(|e| e.to_string())?;
    let buffers = state.lc3_capture_buffers.lock().map_err(|e| e.to_string())?;

    let byte_count = buffers.get(&port).map(|b| b.len()).unwrap_or(0);
    let active = active.contains(&port);

    Ok(Lc3CaptureStatus {
        port,
        byte_count,
        active,
    })
}

#[tauri::command]
pub fn lc3_import_file(path: String) -> Result<Vec<u8>, String> {
    importer::import_lc3_file(&path)
}

#[tauri::command]
pub async fn lc3_decode_and_export(
    data: Vec<u8>,
    sample_rate: u32,
    num_channels: u32,
    bitrate: u32,
    frame_duration_ms: f64,
    hrmode: bool,
    output_dir: String,
    base_name: String,
    save_raw: bool,
    save_wav: bool,
) -> Result<Lc3DecodeResult, String> {
    tokio::task::spawn_blocking(move || {
        let config = DecodeConfig {
            sample_rate,
            num_channels,
            bitrate,
            frame_duration_ms,
            hrmode,
            libpath: None,
        };

        let decoder = Lc3Decoder::new(&config)?;
        let frame_bytes = decoder.get_frame_bytes();
        let frame_samples = decoder.get_frame_samples();

        let data_len = data.len();
        let mut offset = 0;
        let mut frame_count = 0;
        let mut pcm_chunks: Vec<u8> = Vec::new();

        while offset + frame_bytes <= data_len {
            let frame_data = &data[offset..offset + frame_bytes];
            offset += frame_bytes;
            let pcm = decoder.decode_frame(frame_data)?;
            pcm_chunks.extend_from_slice(&pcm);
            frame_count += 1;
        }

        let duration_secs = if sample_rate > 0 {
            frame_count as f64 * frame_samples as f64 / sample_rate as f64
        } else {
            0.0
        };
        let leftover_bytes = data_len - offset;

        let mut saved_files: Vec<String> = Vec::new();

        if save_raw {
            let raw_path = format!("{}\\{}.bin", output_dir, base_name);
            export::save_raw(&data, &raw_path)?;
            saved_files.push(format!("{}.bin", base_name));
        }

        if save_wav && !pcm_chunks.is_empty() {
            let wav_path = format!("{}\\{}.wav", output_dir, base_name);
            export::save_wav(&pcm_chunks, &wav_path, sample_rate, num_channels as u16)?;
            saved_files.push(format!("{}.wav", base_name));
        }

        Ok(Lc3DecodeResult {
            success: true,
            frame_count,
            frame_samples,
            duration_secs,
            leftover_bytes,
            saved_files,
            error: None,
        })
    })
    .await
    .map_err(|e| format!("解码失败: {}", e))?
}
