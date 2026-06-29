use crate::core::config::editor;
use crate::core::xcfg::types::{ConfigItem, ConfigValue};

#[tauri::command]
pub fn get_config_value(
    info_data: Vec<u8>,
    item: ConfigItem,
) -> Result<ConfigValue, String> {
    editor::get_current_value(&info_data, &item)
        .map_err(|e| format!("Failed to get config value: {}", e))
}

#[tauri::command]
pub fn set_config_value(
    info_data: Vec<u8>,
    item: ConfigItem,
    value: ConfigValue,
) -> Result<Vec<u8>, String> {
    let mut data = info_data;
    editor::set_config_value(&mut data, &item, &value)
        .map_err(|e| format!("Failed to set config value: {}", e))?;
    Ok(data)
}

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
