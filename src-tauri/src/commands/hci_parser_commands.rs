use crate::core::hci::parser;
use crate::core::hci::types::ParsedProtocol;

#[tauri::command]
pub fn parse_protocol_line(line: String) -> ParsedProtocol {
    parser::parse_line(&line)
}
