use anyhow::{Result, anyhow};
use crate::core::xcfg::types::{ConfigItem, ConfigValue, EntryType};

pub fn get_current_value(info_data: &[u8], item: &ConfigItem) -> Result<ConfigValue> {
    let off = item.offset as usize;

    match item.entry_type {
        EntryType::TXT => {
            let max_len = if item.str_length > 0 {
                item.str_length as usize
            } else {
                32
            };
            if off + max_len > info_data.len() {
                return Err(anyhow!("TXT offset out of bounds"));
            }
            let raw = &info_data[off..off + max_len];
            let s = String::from_utf8_lossy(raw)
                .trim_end_matches('\0')
                .to_string();
            Ok(ConfigValue::String(s))
        }
        EntryType::MAC => {
            if off + 6 > info_data.len() {
                return Err(anyhow!("MAC offset out of bounds"));
            }
            let mac = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                info_data[off],
                info_data[off + 1],
                info_data[off + 2],
                info_data[off + 3],
                info_data[off + 4],
                info_data[off + 5]
            );
            Ok(ConfigValue::Mac(mac))
        }
        _ => {
            let raw_val = read_raw(info_data, item)?;

            match item.entry_type {
                EntryType::LST | EntryType::LSV => {
                    for opt in &item.options {
                        if opt.value == raw_val {
                            return Ok(ConfigValue::String(opt.label.clone()));
                        }
                    }
                    Ok(ConfigValue::Int(raw_val as i64))
                }
                EntryType::CHK => Ok(ConfigValue::Bool(raw_val != 0)),
                _ => Ok(ConfigValue::Int(raw_val as i64)),
            }
        }
    }
}

fn read_raw(info_data: &[u8], item: &ConfigItem) -> Result<u32> {
    let off = item.offset as usize;

    // Bitfield handling
    if item.bit_width > 0 {
        if off + 4 > info_data.len() {
            return Err(anyhow!("Bitfield offset out of bounds"));
        }
        let val = u32::from_le_bytes([
            info_data[off],
            info_data[off + 1],
            info_data[off + 2],
            info_data[off + 3],
        ]);
        let mask = (1u32 << item.bit_width) - 1;
        return Ok((val >> item.bit_offset) & mask);
    }

    match item.entry_type {
        EntryType::CHK | EntryType::U08 | EntryType::LST => {
            if off + 1 > info_data.len() {
                return Err(anyhow!("U08/CHK/LST offset out of bounds"));
            }
            Ok(info_data[off] as u32)
        }
        EntryType::S08 => {
            if off + 1 > info_data.len() {
                return Err(anyhow!("S08 offset out of bounds"));
            }
            Ok(info_data[off] as i8 as i32 as u32)
        }
        EntryType::U16 => {
            if off + 2 > info_data.len() {
                return Err(anyhow!("U16 offset out of bounds"));
            }
            Ok(u16::from_le_bytes([info_data[off], info_data[off + 1]]) as u32)
        }
        EntryType::UBT => {
            if off + 4 > info_data.len() {
                return Err(anyhow!("UBT offset out of bounds"));
            }
            Ok(u32::from_le_bytes([
                info_data[off],
                info_data[off + 1],
                info_data[off + 2],
                info_data[off + 3],
            ]))
        }
        EntryType::LSV => {
            let vt = item.val_type & 0xFF;
            if vt == 0x10 {
                if off + 2 > info_data.len() {
                    return Err(anyhow!("LSV U16 offset out of bounds"));
                }
                Ok(u16::from_le_bytes([info_data[off], info_data[off + 1]]) as u32)
            } else {
                if off + 4 > info_data.len() {
                    return Err(anyhow!("LSV U32 offset out of bounds"));
                }
                Ok(u32::from_le_bytes([
                    info_data[off],
                    info_data[off + 1],
                    info_data[off + 2],
                    info_data[off + 3],
                ]))
            }
        }
        _ => Err(anyhow!("Unsupported entry type for read_raw")),
    }
}

pub fn set_config_value(info_data: &mut [u8], item: &ConfigItem, value: &ConfigValue) -> Result<()> {
    let off = item.offset as usize;

    match item.entry_type {
        EntryType::TXT => {
            let max_len = if item.str_length > 0 {
                item.str_length as usize
            } else {
                32
            };
            if off + max_len > info_data.len() {
                return Err(anyhow!("TXT offset out of bounds"));
            }
            let s = match value {
                ConfigValue::String(s) => s.clone(),
                _ => return Err(anyhow!("Expected string value for TXT")),
            };
            let bytes = s.as_bytes();
            let len = bytes.len().min(max_len);
            info_data[off..off + len].copy_from_slice(&bytes[..len]);
            // Fill remaining with null
            for i in len..max_len {
                info_data[off + i] = 0;
            }
            Ok(())
        }
        EntryType::MAC => {
            if off + 6 > info_data.len() {
                return Err(anyhow!("MAC offset out of bounds"));
            }
            let mac_str = match value {
                ConfigValue::Mac(s) => s.clone(),
                ConfigValue::String(s) => s.clone(),
                _ => return Err(anyhow!("Expected string value for MAC")),
            };
            let parts: Vec<&str> = mac_str.split(|c| c == ':' || c == '-').collect();
            if parts.len() != 6 {
                return Err(anyhow!("Invalid MAC address format"));
            }
            for (i, part) in parts.iter().enumerate() {
                info_data[off + i] = u8::from_str_radix(part, 16)
                    .map_err(|_| anyhow!("Invalid MAC byte"))?;
            }
            Ok(())
        }
        _ => {
            let int_val = match value {
                ConfigValue::Bool(b) => *b as u32,
                ConfigValue::Int(i) => *i as u32,
                ConfigValue::String(s) => {
                    // Try to find value in options
                    let mut found = None;
                    for opt in &item.options {
                        if opt.label == *s {
                            found = Some(opt.value);
                            break;
                        }
                    }
                    found.unwrap_or_else(|| s.parse::<u32>().unwrap_or(0))
                }
                _ => return Err(anyhow!("Invalid value type")),
            };

            write_raw(info_data, item, int_val)
        }
    }
}

fn write_raw(info_data: &mut [u8], item: &ConfigItem, int_val: u32) -> Result<()> {
    let off = item.offset as usize;

    // Bitfield handling
    if item.bit_width > 0 {
        if off + 4 > info_data.len() {
            return Err(anyhow!("Bitfield offset out of bounds"));
        }
        let mut cur = u32::from_le_bytes([
            info_data[off],
            info_data[off + 1],
            info_data[off + 2],
            info_data[off + 3],
        ]);
        let mask = (1u32 << item.bit_width) - 1;
        cur &= !(mask << item.bit_offset);
        cur |= (int_val & mask) << item.bit_offset;
        let bytes = cur.to_le_bytes();
        info_data[off..off + 4].copy_from_slice(&bytes);
        return Ok(());
    }

    match item.entry_type {
        EntryType::CHK | EntryType::U08 | EntryType::LST => {
            if off + 1 > info_data.len() {
                return Err(anyhow!("U08/CHK/LST offset out of bounds"));
            }
            info_data[off] = (int_val & 0xFF) as u8;
            Ok(())
        }
        EntryType::S08 => {
            if off + 1 > info_data.len() {
                return Err(anyhow!("S08 offset out of bounds"));
            }
            info_data[off] = (int_val as i8) as u8;
            Ok(())
        }
        EntryType::U16 => {
            if off + 2 > info_data.len() {
                return Err(anyhow!("U16 offset out of bounds"));
            }
            let bytes = (int_val as u16).to_le_bytes();
            info_data[off..off + 2].copy_from_slice(&bytes);
            Ok(())
        }
        EntryType::UBT => {
            if off + 4 > info_data.len() {
                return Err(anyhow!("UBT offset out of bounds"));
            }
            let bytes = int_val.to_le_bytes();
            info_data[off..off + 4].copy_from_slice(&bytes);
            Ok(())
        }
        EntryType::LSV => {
            let vt = item.val_type & 0xFF;
            if vt == 0x10 {
                if off + 2 > info_data.len() {
                    return Err(anyhow!("LSV U16 offset out of bounds"));
                }
                let bytes = (int_val as u16).to_le_bytes();
                info_data[off..off + 2].copy_from_slice(&bytes);
            } else {
                if off + 4 > info_data.len() {
                    return Err(anyhow!("LSV U32 offset out of bounds"));
                }
                let bytes = int_val.to_le_bytes();
                info_data[off..off + 4].copy_from_slice(&bytes);
            }
            Ok(())
        }
        _ => Err(anyhow!("Unsupported entry type for write_raw")),
    }
}

pub fn apply_config_changes(
    dcf_data: &mut [u8],
    changes: &[(ConfigItem, ConfigValue)],
    info_offset: usize,
    info_len: usize,
) -> Result<()> {
    let info_start = info_offset + 16;
    let info_end = info_start + info_len;

    if info_end > dcf_data.len() {
        return Err(anyhow!("INFO data extends beyond DCF file"));
    }

    let mut info_data = dcf_data[info_start..info_end].to_vec();

    for (item, value) in changes {
        set_config_value(&mut info_data, item, value)?;
    }

    // Copy back
    dcf_data[info_start..info_end].copy_from_slice(&info_data);

    // Recalculate checksum
    super::checksum::recalculate_checksum(dcf_data, info_offset, info_len);

    Ok(())
}
