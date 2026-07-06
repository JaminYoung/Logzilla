use super::hci_cmd_table;
use super::hci_evt_table;
use super::le_subevt_table;
use super::llcp_table;
use super::lmp_table;
use super::types::{ParsedProtocol, ParamType, ProtocolField};

pub fn parse_line(line: &str) -> ParsedProtocol {
    let line = line.trim();

    if let Some(parsed) = try_parse_hci(line) {
        return parsed;
    }
    if let Some(parsed) = try_parse_lmp(line) {
        return parsed;
    }
    if let Some(parsed) = try_parse_llcp(line) {
        return parsed;
    }

    ParsedProtocol {
        protocol: "???".to_string(),
        name: "???".to_string(),
        opcode_info: String::new(),
        recognized: false,
        fields: vec![],
    }
}

fn try_parse_hci(line: &str) -> Option<ParsedProtocol> {
    let line = line.trim();

    let remaining;
    if line.starts_with('(') {
        let close = line.find(')')?;
        remaining = line[close + 1..].trim();
    } else {
        remaining = line;
    }

    if !remaining.starts_with('[') {
        return None;
    }
    let close_bracket = remaining.find(']')?;
    let rest = remaining[close_bracket + 1..].trim();

    let space_pos = rest.find(' ')?;
    let pkt_type = &rest[..space_pos];
    let after_type = rest[space_pos + 1..].trim();

    if after_type.len() < 2 {
        return None;
    }
    let dir = &after_type[..2];
    if dir != "=>" && dir != "<=" {
        return None;
    }
    let hex_part = after_type[2..].trim();
    let data = parse_hex_bytes(hex_part)?;

    match pkt_type {
        "CMD" => parse_hci_cmd(&data),
        "EVT" => parse_hci_evt(&data),
        "ACL" => Some(ParsedProtocol {
            protocol: "HCI ACL".to_string(),
            name: "ACL_Data".to_string(),
            opcode_info: format!(
                "Handle=0x{:04X}, Length={}",
                if data.len() >= 2 {
                    u16::from_le_bytes([data[0], data[1]]) & 0x0FFF
                } else {
                    0
                },
                if data.len() >= 4 {
                    u16::from_le_bytes([data[2], data[3]])
                } else {
                    0
                }
            ),
            recognized: true,
            fields: vec![],
        }),
        _ => None,
    }
}

fn parse_hci_cmd(data: &[u8]) -> Option<ParsedProtocol> {
    if data.len() < 3 {
        return None;
    }
    let opcode = u16::from_le_bytes([data[0], data[1]]);
    let ogf = (opcode >> 10) as u8;
    let ocf = opcode & 0x03FF;

    let def = hci_cmd_table::lookup(ogf, ocf);
    let name = def
        .map(|d| d.name.to_string())
        .unwrap_or_else(|| format!("Unknown_CMD"));

    let mut fields = if let Some(def) = def {
        parse_fields(&data[3..], def.params, true)
    } else if data.len() > 3 {
        vec![ProtocolField {
            name: "Parameters".to_string(),
            value: format_hex(&data[3..]),
        }]
    } else {
        vec![]
    };

    // Post-process: parse EIR/Advertising data structures
    if ogf == 0x03 && ocf == 0x0052 && data.len() > 4 {
        // Write_Extended_Inquiry_Response: skip FEC_Required(1), parse 240-byte EIR
        let eir_data = &data[4..];
        let eir_fields = parse_ad_structures(eir_data);
        if !eir_fields.is_empty() {
            fields.push(ProtocolField {
                name: "EIR_Structures".to_string(),
                value: String::new(),
            });
            fields.extend(eir_fields);
        }
    } else if ogf == 0x08 && (ocf == 0x0008 || ocf == 0x0009) && data.len() > 4 {
        // LE_Set_Advertising_Data / LE_Set_Scan_Response_Data: skip length(1), parse 31-byte AD
        let ad_data = &data[4..];
        let ad_fields = parse_ad_structures(ad_data);
        if !ad_fields.is_empty() {
            fields.extend(ad_fields);
        }
    } else if ogf == 0x08 && (ocf == 0x0037 || ocf == 0x0038) && data.len() > 5 {
        // LE_Set_Extended_Advertising_Data / LE_Set_Extended_Scan_Response_Data
        // skip handle(1) + op(1) + frag_pref(1) + length(1), parse AD data
        let ad_len = data[6] as usize;
        let ad_start = 7;
        if data.len() > ad_start {
            let ad_data = &data[ad_start..std::cmp::min(ad_start + ad_len, data.len())];
            let ad_fields = parse_ad_structures(ad_data);
            if !ad_fields.is_empty() {
                fields.extend(ad_fields);
            }
        }
    }

    Some(ParsedProtocol {
        protocol: "HCI CMD".to_string(),
        name,
        opcode_info: format!("OGF=0x{:02X}, OCF=0x{:04X}", ogf, ocf),
        recognized: def.is_some(),
        fields,
    })
}

/// Parse AD (Advertising Data) / EIR structures: Length-Type-Data format
fn parse_ad_structures(data: &[u8]) -> Vec<ProtocolField> {
    let mut fields = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        let len = data[offset] as usize;
        if len == 0 {
            break;
        }
        if offset + 1 + len > data.len() {
            break;
        }
        let ad_type = data[offset + 1];
        let ad_data = &data[offset + 2..offset + 1 + len];

        let (type_name, value_str) = match ad_type {
            0x01 => ("Flags".to_string(), format_flags(ad_data)),
            0x02 => ("Incomplete_16bit_UUIDs".to_string(), format_uuid16_list(ad_data)),
            0x03 => ("Complete_16bit_UUIDs".to_string(), format_uuid16_list(ad_data)),
            0x04 => ("Incomplete_32bit_UUIDs".to_string(), format_hex(ad_data)),
            0x05 => ("Complete_32bit_UUIDs".to_string(), format_hex(ad_data)),
            0x06 => ("Incomplete_128bit_UUIDs".to_string(), format_hex(ad_data)),
            0x07 => ("Complete_128bit_UUIDs".to_string(), format_hex(ad_data)),
            0x08 => {
                let name = String::from_utf8_lossy(ad_data).to_string();
                ("Shortened_Local_Name".to_string(), name)
            }
            0x09 => {
                let name = String::from_utf8_lossy(ad_data).to_string();
                ("Complete_Local_Name".to_string(), name)
            }
            0x0A => {
                if !ad_data.is_empty() {
                    ("TX_Power_Level".to_string(), format!("{} dBm", ad_data[0] as i8))
                } else {
                    ("TX_Power_Level".to_string(), String::new())
                }
            }
            0x0D => {
                if ad_data.len() >= 3 {
                    let class = (ad_data[0] as u32) | ((ad_data[1] as u32) << 8) | ((ad_data[2] as u32) << 16);
                    ("Class_of_Device".to_string(), format!("0x{:06X}", class))
                } else {
                    ("Class_of_Device".to_string(), format_hex(ad_data))
                }
            }
            0x0F => ("Service_Data_16bit".to_string(), format_hex(ad_data)),
            0x10 => ("Service_Data_32bit".to_string(), format_hex(ad_data)),
            0x16 => ("Service_Data_16bit_UUID".to_string(), format_hex(ad_data)),
            0x19 => {
                if ad_data.len() >= 2 {
                    let appearance = u16::from_le_bytes([ad_data[0], ad_data[1]]);
                    ("Appearance".to_string(), format!("0x{:04X}", appearance))
                } else {
                    ("Appearance".to_string(), format_hex(ad_data))
                }
            }
            0x20 => ("Service_Data_32bit_UUID".to_string(), format_hex(ad_data)),
            0x21 => ("Service_Data_128bit_UUID".to_string(), format_hex(ad_data)),
            0x24 => ("URI".to_string(), String::from_utf8_lossy(ad_data).to_string()),
            0xFF => {
                if ad_data.len() >= 2 {
                    let company = u16::from_le_bytes([ad_data[0], ad_data[1]]);
                    ("Manufacturer_Specific".to_string(), format!("Company=0x{:04X}, Data={}", company, format_hex(&ad_data[2..])))
                } else {
                    ("Manufacturer_Specific".to_string(), format_hex(ad_data))
                }
            }
            _ => (format!("AD_Type_0x{:02X}", ad_type), format_hex(ad_data)),
        };

        fields.push(ProtocolField {
            name: format!("  [{}] {}", ad_type, type_name),
            value: value_str,
        });

        offset += 1 + len;
    }
    fields
}

fn format_flags(data: &[u8]) -> String {
    if data.is_empty() { return String::new(); }
    let flags = data[0];
    let mut parts = Vec::new();
    if flags & 0x01 != 0 { parts.push("LE_Limited_Disc"); }
    if flags & 0x02 != 0 { parts.push("LE_General_Disc"); }
    if flags & 0x04 != 0 { parts.push("BR/EDR_Not_Supported"); }
    if flags & 0x08 != 0 { parts.push("LE+BR/EDR_Controller"); }
    if flags & 0x10 != 0 { parts.push("LE+BR/EDR_Host"); }
    format!("0x{:02X} ({})", flags, parts.join(", "))
}

fn format_uuid16_list(data: &[u8]) -> String {
    let mut uuids = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let uuid = u16::from_le_bytes([data[i], data[i + 1]]);
        uuids.push(format!("0x{:04X}", uuid));
        i += 2;
    }
    uuids.join(", ")
}

fn parse_hci_evt(data: &[u8]) -> Option<ParsedProtocol> {
    if data.is_empty() {
        return None;
    }
    let code = data[0];

    // Special handling for LE Meta Event (0x3E) — parse subevent
    // HCI event format: Event_Code(1) + Length(1) + Parameters(N)
    // For LE Meta: parameters start at data[2], subevent code is data[2]
    if code == 0x3E && data.len() >= 3 {
        let subevent_code = data[2]; // skip event code(1) + length byte(1)
        let sub_def = le_subevt_table::lookup(subevent_code);
        let sub_name = sub_def
            .map(|d| d.name.to_string())
            .unwrap_or_else(|| format!("Unknown_LE_Subevent"));

        let mut fields = vec![
            ProtocolField { name: "Subevent_Code".to_string(), value: format!("0x{:02X}", subevent_code) },
        ];

        // Subevent parameters start after Subevent_Code: data[3..]
        if let Some(def) = sub_def {
            fields.extend(parse_fields(&data[3..], def.params, sub_def.is_some()));
        } else if data.len() > 3 {
            fields.push(ProtocolField {
                name: "Parameters".to_string(),
                value: format_hex(&data[3..]),
            });
        }

        return Some(ParsedProtocol {
            protocol: "HCI EVT".to_string(),
            name: format!("LE_Meta_Event → {}", sub_name),
            opcode_info: format!("Event Code: 0x3E, Subevent: 0x{:02X}", subevent_code),
            recognized: sub_def.is_some(),
            fields,
        });
    }

    let def = hci_evt_table::lookup(code);
    let name = def
        .map(|d| d.name.to_string())
        .unwrap_or_else(|| format!("Unknown_EVT"));
    let recognized = def.is_some();

    // HCI Event format: Event_Code(1) + Parameter_Length(1) + Event_Parameters(N)
    // Skip the length byte at data[1]; parameters start at data[2]
    let params_start = if data.len() > 1 { 2 } else { data.len() };
    let mut fields = if let Some(def) = def {
        parse_fields(&data[params_start..], def.params, recognized)
    } else if data.len() > params_start {
        vec![ProtocolField {
            name: "Parameters".to_string(),
            value: format_hex(&data[params_start..]),
        }]
    } else {
        vec![]
    };

    // For Command Complete (0x0E) and Command Status (0x0F), resolve the command opcode to name
    // Num_HCI_Command_Packets(1) at data[2], Command_Opcode(2) at data[3..4]
    if (code == 0x0E || code == 0x0F) && data.len() >= 5 {
        let cmd_opcode = u16::from_le_bytes([data[3], data[4]]);
        let ogf = (cmd_opcode >> 10) as u8;
        let ocf = cmd_opcode & 0x03FF;
        let cmd_name = hci_cmd_table::lookup(ogf, ocf)
            .map(|d| d.name.to_string())
            .unwrap_or_else(|| format!("Unknown_CMD(0x{:04X})", cmd_opcode));
        fields.insert(2, ProtocolField {
            name: "Command".to_string(),
            value: format!("{} (OGF=0x{:02X}, OCF=0x{:04X})", cmd_name, ogf, ocf),
        });
    }

    Some(ParsedProtocol {
        protocol: "HCI EVT".to_string(),
        name,
        opcode_info: format!("Event Code: 0x{:02X}", code),
        recognized,
        fields,
    })
}

fn try_parse_lmp(line: &str) -> Option<ParsedProtocol> {
    // Skip timestamp prefix "(xx:xx:xx.xxx) " if present
    let trimmed = line.trim();
    let after_ts = if trimmed.starts_with('(') {
        let close = trimmed.find(')')?;
        trimmed[close + 1..].trim()
    } else {
        trimmed
    };

    let lower = after_ts.to_lowercase();
    let (direction, _rest) = if let Some(r) = lower.strip_prefix("lmp_rx") {
        ("RX", r)
    } else if let Some(r) = lower.strip_prefix("lmp_tx") {
        ("TX", r)
    } else {
        return None;
    };

    // Skip link_id digit(s)
    let rest_orig = &after_ts[6..]; // skip "lmp_rx" or "lmp_tx"
    let chars: Vec<char> = rest_orig.chars().collect();
    if chars.is_empty() || !chars[0].is_ascii_digit() {
        return None;
    }
    let _link_id = chars[0].to_digit(10)? as u8;

    // Find the colon that separates the header from hex data
    let colon_pos = rest_orig.find(':')?;
    let hex_part = rest_orig[colon_pos + 1..].trim();
    let data = parse_hex_bytes(hex_part)?;

    if data.is_empty() {
        return None;
    }

    // LMP format: bit0 = transaction label, bits[7:1] = opcode (high 7 bits)
    let raw_byte = data[0];
    let transaction_label = raw_byte & 1;
    let opcode_or_escape = raw_byte >> 1;

    // Handle LMP escape: when opcode field == 0x7F (127), real opcode is in next byte
    let (opcode, param_start, is_extended) = if opcode_or_escape == 0x7F && data.len() >= 2 {
        (data[1], 2, true)
    } else {
        (opcode_or_escape, 1, false)
    };

    let def = lmp_table::lookup(opcode);
    let name = def
        .map(|d| d.name.to_string())
        .unwrap_or_else(|| format!("Unknown_LMP"));

    let mut fields = vec![ProtocolField {
        name: "Link_ID".to_string(),
        value: _link_id.to_string(),
    }];

    fields.push(ProtocolField {
        name: "Transaction_Label".to_string(),
        value: transaction_label.to_string(),
    });

    if let Some(def) = def {
        fields.extend(parse_fields(&data[param_start..], def.params, true));
    } else if data.len() > param_start {
        fields.push(ProtocolField {
            name: "Parameters".to_string(),
            value: format_hex(&data[param_start..]),
        });
    }

    // Special: LMP_accepted (opcode 3) and LMP_not_accepted (opcode 4) — resolve accepted opcode name
    if opcode == 3 && data.len() > param_start {
        let accepted_opcode = data[param_start];
        let accepted_name = lmp_table::lookup(accepted_opcode)
            .map(|d| d.name.to_string())
            .unwrap_or_else(|| format!("Unknown(0x{:02X})", accepted_opcode));
        fields.push(ProtocolField {
            name: "Accepted_Opcode".to_string(),
            value: format!("{} (0x{:02X})", accepted_name, accepted_opcode),
        });
    }
    if opcode == 4 && data.len() > param_start {
        let rejected_opcode = data[param_start];
        let rejected_name = lmp_table::lookup(rejected_opcode)
            .map(|d| d.name.to_string())
            .unwrap_or_else(|| format!("Unknown(0x{:02X})", rejected_opcode));
        fields.push(ProtocolField {
            name: "Rejected_Opcode".to_string(),
            value: format!("{} (0x{:02X})", rejected_name, rejected_opcode),
        });
    }

    Some(ParsedProtocol {
        protocol: "LMP".to_string(),
        name,
        opcode_info: format!("Opcode: 0x{:02X}, Direction: {}{}", opcode, direction, if is_extended { " (extended)" } else { "" }),
        recognized: def.is_some(),
        fields,
    })
}

fn try_parse_llcp(line: &str) -> Option<ParsedProtocol> {
    // Skip timestamp prefix "(xx:xx:xx.xxx) " if present
    let trimmed = line.trim();
    let after_ts = if trimmed.starts_with('(') {
        let close = trimmed.find(')')?;
        trimmed[close + 1..].trim()
    } else {
        trimmed
    };

    let lower = after_ts.to_lowercase();
    let (direction, _rest) = if let Some(r) = lower.strip_prefix("ll_rx") {
        ("RX", r)
    } else if let Some(r) = lower.strip_prefix("ll_tx") {
        ("TX", r)
    } else {
        return None;
    };

    // Skip link_id digit(s)
    let rest_orig = &after_ts[5..]; // skip "ll_rx" or "ll_tx"
    let chars: Vec<char> = rest_orig.chars().collect();
    if chars.is_empty() || !chars[0].is_ascii_digit() {
        return None;
    }
    let _link_id = chars[0].to_digit(10)? as u8;

    // Find the colon that separates the header from hex data
    let colon_pos = rest_orig.find(':')?;
    let hex_part = rest_orig[colon_pos + 1..].trim();
    let data = parse_hex_bytes(hex_part)?;

    if data.is_empty() {
        return None;
    }

    // First byte is the opcode (payload includes opcode)
    let opcode = data[0];
    let def = llcp_table::lookup(opcode);
    let name = def
        .map(|d| d.name.to_string())
        .unwrap_or_else(|| format!("Unknown_LLCP"));

    let mut fields = vec![ProtocolField {
        name: "Link_ID".to_string(),
        value: _link_id.to_string(),
    }];

    if let Some(def) = def {
        fields.extend(parse_fields(&data[1..], def.params, true));
    } else if data.len() > 1 {
        fields.push(ProtocolField {
            name: "Parameters".to_string(),
            value: format_hex(&data[1..]),
        });
    }

    Some(ParsedProtocol {
        protocol: "LLCP".to_string(),
        name,
        opcode_info: format!("Opcode: 0x{:02X}, Direction: {}", opcode, direction),
        recognized: def.is_some(),
        fields,
    })
}

fn parse_fields(data: &[u8], params: &[(&str, ParamType)], recognized: bool) -> Vec<ProtocolField> {
    let mut fields = Vec::new();
    let mut offset = 0;

    for (name, param_type) in params {
        let (value, consumed) = match param_type {
            ParamType::U8 => {
                if offset >= data.len() {
                    break;
                }
                (format!("0x{:02X}", data[offset]), 1)
            }
            ParamType::U16 => {
                if offset + 2 > data.len() {
                    break;
                }
                let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
                (format!("0x{:04X}", val), 2)
            }
            ParamType::U24 => {
                if offset + 3 > data.len() {
                    break;
                }
                let val = (data[offset] as u32)
                    | ((data[offset + 1] as u32) << 8)
                    | ((data[offset + 2] as u32) << 16);
                (format!("0x{:06X}", val), 3)
            }
            ParamType::U32 => {
                if offset + 4 > data.len() {
                    break;
                }
                let val = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                (format!("0x{:08X}", val), 4)
            }
            ParamType::BdAddr => {
                if offset + 6 > data.len() {
                    break;
                }
                (
                    format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        data[offset + 5],
                        data[offset + 4],
                        data[offset + 3],
                        data[offset + 2],
                        data[offset + 1],
                        data[offset]
                    ),
                    6,
                )
            }
            ParamType::Bytes(n) => {
                let len = if *n == 0 {
                    data.len() - offset
                } else {
                    *n
                };
                if offset + len > data.len() {
                    break;
                }
                (format_hex(&data[offset..offset + len]), len)
            }
        };

        fields.push(ProtocolField {
            name: name.to_string(),
            value,
        });
        offset += consumed;
    }

    // Only show remaining data for unrecognized protocols
    if offset < data.len() && !recognized {
        fields.push(ProtocolField {
            name: "Remaining".to_string(),
            value: format_hex(&data[offset..]),
        });
    }

    fields
}

fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    let hex_str = s.trim();
    if hex_str.is_empty() {
        return Some(Vec::new());
    }
    hex_str
        .split_whitespace()
        .map(|b| u8::from_str_radix(b, 16).ok())
        .collect()
}

fn format_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
