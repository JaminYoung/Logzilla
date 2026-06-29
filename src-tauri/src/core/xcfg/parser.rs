use anyhow::{Result, anyhow};
use super::types::{ConfigItem, ConfigOption, ConfigValue, EntryType};

const ALL_TAGS: [&[u8; 3]; 11] = [
    b"SUB", b"CHK", b"LVL", b"LST", b"LSV",
    b"TXT", b"MAC", b"U08", b"S08", b"U16", b"UBT",
];

fn is_tag(data: &[u8], pos: usize) -> bool {
    if pos + 3 >= data.len() {
        return false;
    }
    let tag = &data[pos..pos + 3];
    ALL_TAGS.iter().any(|t| *t == tag) && data[pos + 3] == 0
}

fn read_utf16_fixed(data: &[u8], pos: usize, nbytes: usize) -> String {
    let end = (pos + nbytes).min(data.len());
    if pos >= end {
        return String::new();
    }
    let raw = &data[pos..end];
    let mut result = String::new();
    let mut i = 0;
    while i + 1 < raw.len() {
        let c = u16::from_le_bytes([raw[i], raw[i + 1]]);
        if c == 0 {
            break;
        }
        if let Some(ch) = char::from_u32(c as u32) {
            result.push(ch);
        }
        i += 2;
    }
    result
}

fn read_utf8_fixed(data: &[u8], pos: usize, nbytes: usize) -> String {
    let end = (pos + nbytes).min(data.len());
    if pos >= end {
        return String::new();
    }
    let raw = &data[pos..end];
    let s = String::from_utf8_lossy(raw);
    s.trim_end_matches('\0').to_string()
}

fn read_u32(data: &[u8], pos: usize) -> u32 {
    if pos + 4 > data.len() {
        return 0;
    }
    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

fn read_u16(data: &[u8], pos: usize) -> u16 {
    if pos + 2 > data.len() {
        return 0;
    }
    u16::from_le_bytes([data[pos], data[pos + 1]])
}

fn read_u8(data: &[u8], pos: usize) -> u8 {
    if pos >= data.len() {
        return 0;
    }
    data[pos]
}

fn read_i8(data: &[u8], pos: usize) -> i8 {
    if pos >= data.len() {
        return 0;
    }
    data[pos] as i8
}

fn read_name_prompt(data: &[u8], pos: usize) -> (String, String, usize) {
    let name = read_utf16_fixed(data, pos, 32);
    let p = pos + 32;
    let prompt_len = read_u32(data, p) as usize;
    let p = p + 4;
    let prompt = read_utf16_fixed(data, p, prompt_len);
    let p = p + prompt_len;
    (name, prompt, p)
}

fn read_name_prompt_var(data: &[u8], pos: usize) -> (String, String, String, usize) {
    let (name, prompt, p) = read_name_prompt(data, pos);
    let var_name = read_utf8_fixed(data, p, 32);
    let p = p + 32;
    (name, prompt, var_name, p)
}

fn read_ui_condition(data: &[u8], pos: usize) -> (Option<String>, usize) {
    if pos >= data.len() {
        return (None, pos);
    }
    let b = data[pos];
    if b == 0 {
        // 单个 0x00 表示无条件
        return (None, pos + 1);
    }
    // 读取 32 字节作为条件变量名
    let var = read_utf8_fixed(data, pos, 32);
    if var.is_empty() {
        // 32 字节全为 0x00 也表示无条件
        return (None, pos + 32);
    }
    (Some(var), pos + 32)
}

fn find_next_sub(data: &[u8], start: usize, end: usize) -> usize {
    let mut p = start;
    while p < end && p + 4 <= data.len() {
        if data[p..p + 3] == *b"SUB" && data[p + 3] == 0 {
            return p;
        }
        p += 1;
    }
    end
}

fn find_next_entry(data: &[u8], start: usize) -> usize {
    let mut p = start;
    while p < data.len().saturating_sub(3) {
        if is_tag(data, p) {
            return p;
        }
        p += 1;
    }
    data.len()
}

fn parse_lst_options(data: &[u8], pos: usize, count: usize) -> (Vec<ConfigOption>, usize) {
    let mut opts = Vec::new();
    let mut p = pos;
    for i in 0..count {
        let label = read_utf16_fixed(data, p, 32);
        opts.push(ConfigOption {
            label,
            value: i as u32,
        });
        p += 32;
    }
    (opts, p)
}

fn parse_lsv_options(data: &[u8], pos: usize, count: usize) -> (Vec<ConfigOption>, usize) {
    let mut opts = Vec::new();
    let mut p = pos;
    for _ in 0..count {
        let label = read_utf16_fixed(data, p, 32);
        p += 32;
        let val = read_u32(data, p);
        p += 4;
        opts.push(ConfigOption {
            label,
            value: val,
        });
    }
    (opts, p)
}

pub fn parse_entries(data: &[u8], start: usize, end: usize) -> Result<Vec<ConfigItem>> {
    let mut items = Vec::new();
    let mut pos = start;

    while pos < end && pos + 4 <= data.len() {
        if !is_tag(data, pos) {
            pos += 1;
            continue;
        }

        let entry_type_str = std::str::from_utf8(&data[pos..pos + 3])
            .map_err(|_| anyhow!("Invalid UTF-8 in tag"))?;
        let content = pos + 4;

        match entry_type_str {
            "LVL" => {
                let level_val = read_u32(data, content);
                items.push(ConfigItem {
                    entry_type: EntryType::LVL,
                    level_value: level_val,
                    ..Default::default()
                });
                pos = content + 4;
            }
            "SUB" => {
                let (name, prompt, p) = read_name_prompt(data, content);
                // Skip padding (32 bytes)
                let p = p + 32;
                let next_sub = find_next_sub(data, p, end);
                let children = parse_entries(data, p, next_sub)?;
                items.push(ConfigItem {
                    entry_type: EntryType::SUB,
                    label_cn: name.clone(),
                    name: name,
                    tooltip: prompt,
                    children,
                    ..Default::default()
                });
                pos = next_sub;
            }
            "CHK" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let default_val = read_u8(data, p);
                let p = p + 1; // default_val 只有 1 字节
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::CHK,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    default_value: Some(ConfigValue::Bool(default_val != 0)),
                    value: Some(ConfigValue::Bool(default_val != 0)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "LST" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let opt_count = read_u8(data, p) as usize;
                let p = p + 1;
                let (opts, p) = parse_lst_options(data, p, opt_count);
                let default_idx = read_u8(data, p);
                let p = p + 32; // Skip default_val 32 bytes
                let (cond, p) = read_ui_condition(data, p);
                let sel = default_idx as usize;
                let val = if sel < opts.len() {
                    opts[sel].label.clone()
                } else {
                    String::new()
                };
                items.push(ConfigItem {
                    entry_type: EntryType::LST,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    options: opts,
                    default_value: Some(ConfigValue::Int(default_idx as i64)),
                    value: Some(ConfigValue::String(val)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "LSV" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let val_type = read_u16(data, p);
                let p = p + 2;
                let default_val = read_u32(data, p);
                let p = p + 4;
                let opt_count = read_u8(data, p) as usize;
                let p = p + 1;
                let (opts, p) = parse_lsv_options(data, p, opt_count);
                let (cond, p) = read_ui_condition(data, p);

                // Find selected index
                let _sel = opts.iter().position(|o| o.value == default_val).unwrap_or(0);

                // Calculate max_val for bitfield types
                let max_val: i32 = if (val_type & 0xFF) == 0x20 {
                    (1 << ((val_type >> 8) & 0xFF)) - 1
                } else {
                    0
                };

                items.push(ConfigItem {
                    entry_type: EntryType::LSV,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    val_type,
                    options: opts,
                    default_value: Some(ConfigValue::Int(default_val as i64)),
                    value: Some(ConfigValue::Int(default_val as i64)),
                    min_val: 0,
                    max_val,
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "U08" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let mn = read_u8(data, p) as i32;
                let mx = read_u8(data, p + 1) as i32;
                let dv = read_u8(data, p + 2) as i32;
                let p = p + 3;
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::U08,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    min_val: mn,
                    max_val: mx,
                    default_value: Some(ConfigValue::Int(dv as i64)),
                    value: Some(ConfigValue::Int(dv as i64)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "S08" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let mn = read_i8(data, p) as i32;
                let mx = read_i8(data, p + 1) as i32;
                let dv = read_i8(data, p + 2) as i32;
                let p = p + 3;
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::S08,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    min_val: mn,
                    max_val: mx,
                    default_value: Some(ConfigValue::Int(dv as i64)),
                    value: Some(ConfigValue::Int(dv as i64)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "U16" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let mn = read_u16(data, p) as i32;
                let mx = read_u16(data, p + 2) as i32;
                let dv = read_u16(data, p + 4) as i32;
                let p = p + 6;
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::U16,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    min_val: mn,
                    max_val: mx,
                    default_value: Some(ConfigValue::Int(dv as i64)),
                    value: Some(ConfigValue::Int(dv as i64)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "UBT" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let bit_num = read_u8(data, p);
                let mn = read_u32(data, p + 1) as i32;
                let mx = read_u32(data, p + 5) as i32;
                let dv = read_u32(data, p + 9) as i32;
                let p = p + 13;
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::UBT,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    bit_width: bit_num,
                    min_val: mn,
                    max_val: mx,
                    default_value: Some(ConfigValue::Int(dv as i64)),
                    value: Some(ConfigValue::Int(dv as i64)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "MAC" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let length = read_u8(data, p);
                let p = p + 1;
                let mut addr_start = [0u8; 6];
                let mut addr_end = [0u8; 6];
                if p + 12 <= data.len() {
                    addr_start.copy_from_slice(&data[p..p + 6]);
                    addr_end.copy_from_slice(&data[p + 6..p + 12]);
                }
                let p = p + 12;

                // Try to read addr_set
                let next_off = find_next_entry(data, p);
                let next_off = if next_off > end { end } else { next_off };
                let mut addr_set = None;
                let mut p = p;
                if next_off > p && next_off - p >= 6 {
                    if data[p] != 0 {
                        let mut addr = [0u8; 6];
                        addr.copy_from_slice(&data[p..p + 6]);
                        addr_set = Some(addr);
                        p += 6;
                    }
                }

                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::MAC,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    str_length: length,
                    addr_start: Some(addr_start),
                    addr_end: Some(addr_end),
                    addr_set,
                    default_value: Some(ConfigValue::Mac(format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        addr_start[0], addr_start[1], addr_start[2],
                        addr_start[3], addr_start[4], addr_start[5]
                    ))),
                    value: Some(ConfigValue::Mac(format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        addr_start[0], addr_start[1], addr_start[2],
                        addr_start[3], addr_start[4], addr_start[5]
                    ))),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            "TXT" => {
                let (name, prompt, var_name, p) = read_name_prompt_var(data, content);
                let length = read_u8(data, p);
                let p = p + 1;
                let default_str = read_utf16_fixed(data, p, 32);
                let p = p + 32;
                let (cond, p) = read_ui_condition(data, p);
                items.push(ConfigItem {
                    entry_type: EntryType::TXT,
                    label_cn: name.clone(),
                    name: if var_name.is_empty() { name.clone() } else { var_name.clone() },
                    tooltip: prompt,
                    var_name: var_name,
                    str_length: length,
                    default_value: Some(ConfigValue::String(default_str.clone())),
                    value: Some(ConfigValue::String(default_str)),
                    ui_condition_var: cond,
                    ..Default::default()
                });
                pos = p;
            }
            _ => {
                // Unknown tag, skip to next entry
                pos = find_next_entry(data, pos + 4);
            }
        }
    }

    Ok(items)
}

pub fn parse_full_xcfg(
    data: &[u8],
    info_offset: usize,
    info_len: usize,
) -> Result<(Vec<ConfigItem>, Vec<u8>)> {
    let config_tree_offset = 0x50; // Default config tree start
    let info_data = data[info_offset + 16..info_offset + 16 + info_len].to_vec();
    let tree = parse_entries(data, config_tree_offset, info_offset)?;
    Ok((tree, info_data))
}
