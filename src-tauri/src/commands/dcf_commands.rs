use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::core::dcf;
use crate::core::xcfg::{parser as xcfg_parser, offset, types::ConfigItem};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfInfo {
    pub path: String,
    pub header: dcf::parser::DcfHeader,
    pub config_tree: Vec<ConfigItem>,
    pub info_offset: usize,
    pub info_len: usize,
    pub info_data: Vec<u8>,
}

#[tauri::command]
pub fn open_dcf(path: String, state: State<AppState>) -> Result<DcfInfo, String> {
    info!("Opening DCF file: {}", path);
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    info!("DCF file size: {} bytes", data.len());

    let header = dcf::parser::parse_dcf(&data)
        .map_err(|e| format!("Failed to parse DCF: {}", e))?;
    info!("DCF header parsed: version={}, info_offset={}, info_len={}", 
          header.version, header.info_offset, header.info_len);

    let info_offset = header.info_offset;
    let info_len = header.info_len as usize;

    info!("Parsing config tree from offset 0x{:X} to 0x{:X}", 0x50, info_offset);
    let (mut config_tree, info_data) = xcfg_parser::parse_full_xcfg(&data, info_offset, info_len)
        .map_err(|e| format!("Failed to parse config tree: {}", e))?;
    info!("Parsed {} config items", config_tree.len());

    // Assign offsets
    offset::assign_offsets(&mut config_tree);
    info!("Assigned offsets to config items");

    // Update state
    {
        let mut dcf_data = state.dcf_data.lock().map_err(|e| format!("Lock error: {}", e))?;
        *dcf_data = Some(data);

        let mut dcf_path = state.dcf_path.lock().map_err(|e| format!("Lock error: {}", e))?;
        *dcf_path = Some(path.clone());

        let mut tree = state.config_tree.lock().map_err(|e| format!("Lock error: {}", e))?;
        *tree = config_tree.clone();

        let mut info = state.info_data.lock().map_err(|e| format!("Lock error: {}", e))?;
        *info = info_data.clone();

        let mut off = state.info_offset.lock().map_err(|e| format!("Lock error: {}", e))?;
        *off = info_offset;

        let mut len = state.info_len.lock().map_err(|e| format!("Lock error: {}", e))?;
        *len = info_len;
    }

    info!("DCF file loaded successfully");
    Ok(DcfInfo {
        path,
        header,
        config_tree,
        info_offset,
        info_len,
        info_data,
    })
}
