use serde::{Deserialize, Serialize};
use crate::core::usb_audio::extractor::{self, SplitMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbAudioAnalysis {
    pub file_name: String,
    pub total_packets: usize,
    pub iso_packets: usize,
    pub interfaces: Vec<InterfaceInfo>,
    pub set_interface_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    pub interface: u8,
    pub endpoint: u8,
    pub direction: String,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub channels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResultDto {
    pub segment_count: usize,
    pub files: Vec<String>,
}

#[tauri::command]
pub async fn usb_audio_analyze(path: String) -> Result<UsbAudioAnalysis, String> {
    let result = tokio::task::spawn_blocking(move || {
        let analysis = extractor::analyze(&path)?;
        Ok::<_, String>(UsbAudioAnalysis {
            file_name: analysis.file_name,
            total_packets: analysis.total_packets,
            iso_packets: analysis.iso_packets,
            interfaces: analysis.interfaces.iter().map(|i| InterfaceInfo {
                interface: i.interface,
                endpoint: i.endpoint,
                direction: i.direction.clone(),
                sample_rate: i.sample_rate,
                bit_depth: i.bit_depth,
                channels: i.channels,
            }).collect(),
            set_interface_count: analysis.set_interface_count,
        })
    })
    .await
    .map_err(|e| format!("分析失败: {}", e))??;
    Ok(result)
}

#[tauri::command]
pub async fn usb_audio_extract(
    path: String,
    split_mode: String,
    threshold_ms: u64,
    output_dir: String,
) -> Result<ExtractResultDto, String> {
    let mode = match split_mode.as_str() {
        "A" => SplitMode::A,
        "B" => SplitMode::B,
        "C" => SplitMode::C,
        _ => return Err("无效的分段模式".to_string()),
    };

    let result = tokio::task::spawn_blocking(move || {
        let extract = extractor::extract(&path, mode, threshold_ms, &output_dir)?;
        Ok::<_, String>(ExtractResultDto {
            segment_count: extract.segment_count,
            files: extract.files,
        })
    })
    .await
    .map_err(|e| format!("提取失败: {}", e))??;
    Ok(result)
}
