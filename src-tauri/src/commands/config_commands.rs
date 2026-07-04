use crate::core::config::editor;
use crate::core::xcfg::types::{ConfigItem, ConfigValue};

#[tauri::command]
pub fn apply_config_changes(
    dcf_data: Vec<u8>,
    changes: Vec<(ConfigItem, ConfigValue)>,
    info_offset: usize,
    info_len: usize,
) -> Result<Vec<u8>, String> {
    let mut data = dcf_data;
    editor::apply_config_changes(&mut data, &changes, info_offset, info_len)
        .map_err(|e| format!("Failed to apply config changes: {}", e))?;
    Ok(data)
}
