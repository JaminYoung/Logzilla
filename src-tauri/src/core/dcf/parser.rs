use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

const DCF_MAGIC: [u8; 4] = [0x44, 0x43, 0x46, 0x00]; // "DCF\0"
const INFO_MARKER: [u8; 4] = [0x49, 0x4E, 0x46, 0x4F]; // "INFO"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfHeader {
    pub magic: [u8; 4],
    pub data_size: u32,
    pub timestamp: u32,
    pub company: [u8; 4],
    pub version: u32,
    pub config_tree_offset: u32,
    pub info_offset: usize,
    pub info_version: u16,
    pub info_len: u16,
    pub code_version: u16,
}

pub fn parse_dcf(data: &[u8]) -> Result<DcfHeader> {
    if data.len() < 48 {
        return Err(anyhow!("DCF file too small: {} bytes", data.len()));
    }

    let magic = [data[0], data[1], data[2], data[3]];
    if magic != DCF_MAGIC {
        return Err(anyhow!("Invalid DCF magic: {:02X?}", magic));
    }

    let data_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let timestamp = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let company = [data[12], data[13], data[14], data[15]];
    let version = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let config_tree_offset = u32::from_le_bytes([data[0x30], data[0x31], data[0x32], data[0x33]]);

    // 查找 INFO marker
    let info_offset = find_info_marker(data)?;

    // 解析 INFO header
    if info_offset + 16 > data.len() {
        return Err(anyhow!("INFO header extends beyond file"));
    }

    let flag = u32::from_le_bytes([
        data[info_offset],
        data[info_offset + 1],
        data[info_offset + 2],
        data[info_offset + 3],
    ]);

    if flag != 0x4F464E49 {
        // "INFO" reversed
        return Err(anyhow!("Invalid INFO flag: {:08X}", flag));
    }

    let info_version = u16::from_le_bytes([data[info_offset + 4], data[info_offset + 5]]);
    let info_len = u16::from_le_bytes([data[info_offset + 6], data[info_offset + 7]]);
    let code_version = u16::from_le_bytes([data[info_offset + 8], data[info_offset + 9]]);

    Ok(DcfHeader {
        magic,
        data_size,
        timestamp,
        company,
        version,
        config_tree_offset,
        info_offset,
        info_version,
        info_len,
        code_version,
    })
}

pub fn find_info_marker(data: &[u8]) -> Result<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if data[i..i + 4] == INFO_MARKER {
            return Ok(i);
        }
    }
    Err(anyhow!("INFO marker not found in DCF file"))
}

pub fn read_info_data(data: &[u8], info_offset: usize, info_len: usize) -> Result<Vec<u8>> {
    let info_start = info_offset + 16;
    let info_end = info_start + info_len;

    if info_end > data.len() {
        return Err(anyhow!("INFO data extends beyond file"));
    }

    Ok(data[info_start..info_end].to_vec())
}
