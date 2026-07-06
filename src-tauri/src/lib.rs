use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

mod commands;
mod core;

use crate::core::serial::port::SerialPortHandle;
use commands::{
    config_commands, dcf_commands, file_commands, hci_commands, hci_parser_commands, lc3_commands,
    live_import_commands, search_commands, serial_commands, usb_audio_commands,
};

pub struct SerialReaderConfig {
    pub timestamp_enabled: bool,
    pub max_lines: usize,
    pub panel: usize,
}

pub struct SerialReaderControl {
    pub stop: Arc<AtomicBool>,
    pub config: Arc<Mutex<SerialReaderConfig>>,
}

pub struct AppState {
    pub dcf_data: Mutex<Option<Vec<u8>>>,
    pub dcf_path: Mutex<Option<String>>,
    pub config_tree: Mutex<Vec<core::xcfg::types::ConfigItem>>,
    pub info_data: Mutex<Vec<u8>>,
    pub info_offset: Mutex<usize>,
    pub info_len: Mutex<usize>,
    pub serial_ports: Mutex<HashMap<String, Arc<SerialPortHandle>>>,
    pub serial_readers: Mutex<HashMap<String, SerialReaderControl>>,
    pub connected_ports: Mutex<Vec<String>>,
    pub lc3_capture_buffers: Mutex<HashMap<String, Vec<u8>>>,
    pub lc3_capture_active: Mutex<Vec<String>>,
    pub logs: Mutex<Vec<String>>,
    pub logs2: Mutex<Vec<String>>,
    pub logs_version: Mutex<u64>,
    pub logs2_version: Mutex<u64>,
    /// Number of lines dropped from the front of each backend log buffer.
    /// The frontend sends absolute line cursors; backend maps them with
    /// `absolute_line - base` to locate the current buffer index.
    pub logs_base: Mutex<usize>,
    pub logs2_base: Mutex<usize>,
    /// Incomplete line buffers used by the legacy one-shot read command.
    pub partial_lines: Mutex<HashMap<String, String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            dcf_data: Mutex::new(None),
            dcf_path: Mutex::new(None),
            config_tree: Mutex::new(Vec::new()),
            info_data: Mutex::new(Vec::new()),
            info_offset: Mutex::new(0),
            info_len: Mutex::new(0),
            serial_ports: Mutex::new(HashMap::new()),
            serial_readers: Mutex::new(HashMap::new()),
            connected_ports: Mutex::new(Vec::new()),
            lc3_capture_buffers: Mutex::new(HashMap::new()),
            lc3_capture_active: Mutex::new(Vec::new()),
            logs: Mutex::new(Vec::new()),
            logs2: Mutex::new(Vec::new()),
            logs_version: Mutex::new(0),
            logs2_version: Mutex::new(0),
            logs_base: Mutex::new(0),
            logs2_base: Mutex::new(0),
            partial_lines: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            dcf_commands::open_dcf,
            serial_commands::list_serial_ports,
            serial_commands::open_serial_port,
            serial_commands::close_serial_port,
            serial_commands::send_serial_data,
            serial_commands::start_serial_reader,
            serial_commands::stop_serial_reader,
            serial_commands::read_serial_data,
            config_commands::apply_config_changes,
            file_commands::write_file,
            file_commands::read_file,
            hci_commands::extract_hci,
            hci_commands::reveal_in_explorer,
            live_import_commands::live_import_init,
            live_import_commands::live_import_send_frame,
            live_import_commands::live_import_is_ready,
            live_import_commands::live_import_stats,
            live_import_commands::live_import_close,
            live_import_commands::live_import_is_fts_running,
            live_import_commands::launch_wps,
            lc3_commands::lc3_get_connected_ports,
            lc3_commands::lc3_start_capture,
            lc3_commands::lc3_stop_capture,
            lc3_commands::lc3_get_capture_status,
            lc3_commands::lc3_import_file,
            lc3_commands::lc3_decode_and_export,
            usb_audio_commands::usb_audio_analyze,
            usb_audio_commands::usb_audio_extract,
            hci_parser_commands::parse_protocol_line,
            search_commands::get_logs,
            search_commands::clear_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_dcf_parse() {
        let path = "../../app.dcf";
        let data = fs::read(path).expect("Failed to read DCF file");
        println!("DCF file size: {} bytes", data.len());

        let header = core::dcf::parser::parse_dcf(&data).expect("Failed to parse DCF header");
        println!(
            "Header: version={}, info_offset={}, info_len={}",
            header.version, header.info_offset, header.info_len
        );

        let info_offset = header.info_offset;
        let info_len = header.info_len as usize;

        let (mut config_tree, _info_data) =
            core::xcfg::parser::parse_full_xcfg(&data, info_offset, info_len)
                .expect("Failed to parse config tree");

        // Assign offsets
        core::xcfg::offset::assign_offsets(&mut config_tree);

        println!("\n=== Config Tree ===");
        print_config_tree(&config_tree, 0);
    }

    fn print_config_tree(items: &[core::xcfg::types::ConfigItem], indent: usize) {
        let prefix = "  ".repeat(indent);
        for item in items {
            match item.entry_type {
                core::xcfg::types::EntryType::SUB => {
                    println!("{}SUB: {}", prefix, item.label_cn);
                    print_config_tree(&item.children, indent + 1);
                }
                core::xcfg::types::EntryType::LVL => {
                    println!("{}LVL: level_value={}", prefix, item.level_value);
                }
                _ => {
                    let cond_str = match &item.ui_condition_var {
                        Some(var) => format!("[condition={}]", var),
                        None => String::new(),
                    };
                    let val_str = match &item.value {
                        Some(v) => format!("{:?}", v),
                        None => "None".to_string(),
                    };
                    println!(
                        "{}{}: {} (var={}) val={} {} {}",
                        prefix,
                        format!("{:?}", item.entry_type),
                        item.label_cn,
                        item.var_name,
                        val_str,
                        format!("min={} max={}", item.min_val, item.max_val),
                        cond_str
                    );
                }
            }
        }
    }
}
