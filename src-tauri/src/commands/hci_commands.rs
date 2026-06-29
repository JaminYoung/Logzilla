use std::io::Write;

struct HciPacket {
    timestamp_usec: u64,
    flags: u32,
    data: Vec<u8>,
}

fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    let hex_str = s.trim();
    if hex_str.is_empty() {
        return Ok(Vec::new());
    }
    hex_str
        .split_whitespace()
        .map(|b| u8::from_str_radix(b, 16).map_err(|_| format!("invalid hex byte: {}", b)))
        .collect()
}

/// Offset in microseconds between btsnoop epoch (Jan 1, 0 AD) and Unix epoch (Jan 1, 1970).
///
/// The btsnoop format defines timestamps as microseconds since midnight January 1, 0 AD
/// (nominal Gregorian).  The offset from Unix epoch to O AD is computed via chrono.
fn btsnoop_epoch_offset_usec() -> u64 {
    let unix_epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .and_hms_milli_opt(0, 0, 0, 0)
        .unwrap();
    let ad_epoch = chrono::NaiveDate::from_ymd_opt(0, 1, 1)
        .unwrap()
        .and_hms_milli_opt(0, 0, 0, 0)
        .unwrap();
    (unix_epoch - ad_epoch).num_microseconds().unwrap() as u64
}

fn unix_usec_to_btsnoop(unix_usec: u64) -> u64 {
    static OFFSET: std::sync::LazyLock<u64> =
        std::sync::LazyLock::new(btsnoop_epoch_offset_usec);
    unix_usec + *OFFSET
}

/// Convert a local-time `(HH:MM:SS.mmm)` into btsnoop epoch microseconds.
fn local_pc_timestamp_to_btsnoop(h: u32, m: u32, s: u32, ms: u32) -> u64 {
    let now_local = chrono::Local::now();
    let date = now_local.date_naive();
    let local_naive = chrono::NaiveDateTime::new(
        date,
        chrono::NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap(),
    );
    let utc_dt = local_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let unix_usec = utc_dt.timestamp_micros() as u64;
    unix_usec_to_btsnoop(unix_usec)
}

/// Convert a chip-uptime `[HH:MM:SS.mmm]` into btsnoop epoch microseconds
/// using today's UTC midnight as the reference.
fn chip_timestamp_to_btsnoop(h: u32, m: u32, s: u32, ms: u32) -> u64 {
    let now_utc = chrono::Utc::now();
    let date = now_utc.date_naive();
    let chip_naive = chrono::NaiveDateTime::new(
        date,
        chrono::NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap(),
    );
    let unix_usec = chip_naive.and_utc().timestamp_micros() as u64;
    unix_usec_to_btsnoop(unix_usec)
}

fn write_btsnoop<W: Write>(writer: &mut W, packets: &[HciPacket]) -> Result<(), String> {
    writer.write_all(b"btsnoop\0").map_err(|e| e.to_string())?;
    writer.write_all(&1u32.to_be_bytes()).map_err(|e| e.to_string())?;
    // datalink = 1002 (HCI UART / H4) — Frontline expects 1002 for H4
    writer.write_all(&1002u32.to_be_bytes()).map_err(|e| e.to_string())?;

    for pkt in packets {
        let packet_len = pkt.data.len() as u32;

        writer.write_all(&packet_len.to_be_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(&packet_len.to_be_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(&pkt.flags.to_be_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(&0u32.to_be_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(&pkt.timestamp_usec.to_be_bytes()).map_err(|e| e.to_string())?;
        writer.write_all(&pkt.data).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Parse a single HCI log line, prepending the H4 type indicator byte.
///
/// Accepts:
///   (15:20:52.807) [00:00:19.017] CMD => 03 0c 00    → data: 01 03 0c 00
///   [00:00:19.017] CMD => 03 0c 00                    → data: 01 03 0c 00
///
/// H4 mapping: CMD → 0x01, ACL → 0x02, EVT → 0x04
fn parse_hci_line(line: &str) -> Option<(u64, u32, Vec<u8>)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // ---- optional PC timestamp (HH:MM:SS.mmm) at line start ----
    let remaining: &str;
    let pc_ts: Option<(u32, u32, u32, u32)>;

    if line.starts_with('(') {
        let close_paren = line.find(')')?;
        let ts_str = &line[1..close_paren];
        if ts_str.len() != 12 {
            return None;
        }
        let h: u32 = ts_str[0..2].parse().ok()?;
        let m: u32 = ts_str[3..5].parse().ok()?;
        let s: u32 = ts_str[6..8].parse().ok()?;
        let ms: u32 = ts_str[9..12].parse().ok()?;
        pc_ts = Some((h, m, s, ms));
        remaining = line[close_paren + 1..].trim_start();
    } else {
        pc_ts = None;
        remaining = line;
    }

    // ---- mandatory chip timestamp [HH:MM:SS.mmm] ----
    if !remaining.starts_with('[') {
        return None;
    }
    let close_bracket = remaining.find(']')?;
    let ts_part = &remaining[1..close_bracket];
    if ts_part.len() != 12 {
        return None;
    }
    let chip_h: u32 = ts_part[0..2].parse().ok()?;
    let chip_m: u32 = ts_part[3..5].parse().ok()?;
    let chip_s: u32 = ts_part[6..8].parse().ok()?;
    let chip_ms: u32 = ts_part[9..12].parse().ok()?;

    let rest = remaining[close_bracket + 1..].trim_start();
    if rest.len() < 6 {
        return None;
    }

    // Packet type
    let space_after_type = rest.find(' ')?;
    let pkt_type = &rest[..space_after_type];

    let after_type = rest[space_after_type + 1..].trim_start();

    // Direction
    if after_type.len() < 2 {
        return None;
    }
    let dir_str = &after_type[..2];
    let direction_sent = match dir_str {
        "=>" => true,
        "<=" => false,
        _ => return None,
    };

    // Hex data (without H4 prefix)
    let hex_part = after_type[2..].trim();
    if hex_part.is_empty() {
        return None;
    }
    let raw_bytes = parse_hex_bytes(hex_part).ok()?;
    if raw_bytes.is_empty() {
        return None;
    }

    // Determine flags and H4 type indicator
    // 0 = HCI Command (sent), 1 = HCI Event (received),
    // 2 = ACL Data (sent), 3 = ACL Data (received)
    let (flags, h4_type): (u32, u8) = match (pkt_type, direction_sent) {
        ("CMD", true) => (0, 0x01),
        ("EVT", false) => (1, 0x04),
        ("ACL", true) => (2, 0x02),
        ("ACL", false) => (3, 0x02),
        _ => return None,
    };

    // Prepend H4 type indicator to packet data
    let mut data_with_h4 = Vec::with_capacity(1 + raw_bytes.len());
    data_with_h4.push(h4_type);
    data_with_h4.extend_from_slice(&raw_bytes);

    // Timestamp
    let timestamp = match pc_ts {
        Some((h, m, s, ms)) => local_pc_timestamp_to_btsnoop(h, m, s, ms),
        None => chip_timestamp_to_btsnoop(chip_h, chip_m, chip_s, chip_ms),
    };

    Some((timestamp, flags, data_with_h4))
}

#[tauri::command]
pub fn extract_hci(input_path: String) -> Result<String, String> {
    let content = std::fs::read_to_string(&input_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    let mut packets: Vec<HciPacket> = Vec::new();

    for line in content.lines() {
        if let Some((ts, flags, data)) = parse_hci_line(line) {
            packets.push(HciPacket {
                timestamp_usec: ts,
                flags,
                data,
            });
        }
    }

    if packets.is_empty() {
        return Err("未找到任何 HCI 数据包".to_string());
    }

    let output_path = if input_path.ends_with(".txt") || input_path.ends_with(".log") {
        let dot = input_path.rfind('.').unwrap();
        format!("{}.cfa", &input_path[..dot])
    } else {
        format!("{}.cfa", input_path)
    };

    let file = std::fs::File::create(&output_path)
        .map_err(|e| format!("无法创建输出文件: {}", e))?;
    let mut writer = std::io::BufWriter::new(file);
    write_btsnoop(&mut writer, &packets)?;

    Ok(format!(
        "成功提取 {} 个 HCI 数据包\n保存到: {}",
        packets.len(),
        output_path
    ))
}

#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg("/select,")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("无法打开文件管理器: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_from_sample_log() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sample_path = manifest_dir.join("..").join("COM6_2026-05-23_15-20-33.txt");
        if !sample_path.exists() {
            eprintln!("Sample log not found at {:?}, skipping", sample_path);
            return;
        }
        let result = extract_hci(sample_path.to_str().unwrap().to_string());
        assert!(result.is_ok(), "extract_hci failed: {:?}", result.err());
        let msg = result.unwrap();
        assert!(msg.contains("个 HCI 数据包"), "Unexpected result: {}", msg);
        // Verify .cfa file was created
        let cfa_path = sample_path.with_extension("cfa");
        assert!(cfa_path.exists(), "CFA file not created at {:?}", cfa_path);
        // Read and verify btsnoop format
        let data = std::fs::read(&cfa_path).expect("Failed to read CFA");
        assert!(data.len() > 16, "CFA file too small");
        assert_eq!(&data[0..8], b"btsnoop\0", "Invalid btsnoop magic");
        // Version = 1
        let mut ver = [0u8; 4];
        ver.copy_from_slice(&data[8..12]);
        assert_eq!(u32::from_be_bytes(ver), 1, "Wrong btsnoop version");
        // Datalink
        let mut dl = [0u8; 4];
        dl.copy_from_slice(&data[12..16]);
        let datalink = u32::from_be_bytes(dl);
        assert_eq!(datalink, 1002, "Expected datalink 1002 (H4), got {}", datalink);
        // Count packets: each packet has 4 words (16 bytes) header + data
        let pkt_hdr_size = 24usize; // 4+4+4+4+8
        let mut pos = 16;
        let mut pkt_count = 0u32;
        while pos + pkt_hdr_size <= data.len() {
            let mut orig_len = [0u8; 4];
            orig_len.copy_from_slice(&data[pos..pos+4]);
            let orig = u32::from_be_bytes(orig_len);
            // included_len at pos+4..pos+8, flags at pos+8..pos+12, drops at pos+12..pos+16, ts at pos+16..pos+24
            let mut inc_len = [0u8; 4];
            inc_len.copy_from_slice(&data[pos+4..pos+8]);
            let inc = u32::from_be_bytes(inc_len);
            assert_eq!(orig, inc, "original_len != included_len for packet {}", pkt_count);
            let mut flags_buf = [0u8; 4];
            flags_buf.copy_from_slice(&data[pos+8..pos+12]);
            let flags = u32::from_be_bytes(flags_buf);
            assert!(flags <= 3, "Invalid flags {} for packet {}", flags, pkt_count);
            let pkt_data_len = orig as usize;
            assert!(pos + pkt_hdr_size + pkt_data_len <= data.len(), "Packet data exceeds file");
            // First byte of packet data is the H4 type indicator (0x01/0x02/0x04)
            pos += pkt_hdr_size + pkt_data_len;
            pkt_count += 1;
        }
        assert!(pkt_count > 0, "No packets found in CFA file");
        // Cleanup
        let _ = std::fs::remove_file(&cfa_path);
    }

    #[test]
    fn test_parse_hci_cmd() {
        let line = "[00:00:00.107] CMD => 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_ts, flags, data) = result.unwrap();
        assert_eq!(flags, 0);
        assert_eq!(&data, &[0x01, 0x03, 0x0c, 0x00]);
    }

    #[test]
    fn test_parse_hci_evt() {
        let line = "[00:00:00.128] EVT <= 0e 04 05 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_ts, flags, data) = result.unwrap();
        assert_eq!(flags, 1);
        assert_eq!(data[0], 0x04, "EVT should have H4 type 0x04");
    }

    #[test]
    fn test_parse_hci_acl_sent() {
        let line = "[00:00:01.140] ACL => 80 00 10 00 0c 00 01 00 0b 02 08 00 02 00 00 00 80 02 00 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_ts, flags, data) = result.unwrap();
        assert_eq!(flags, 2);
        assert_eq!(data[0], 0x02, "ACL should have H4 type 0x02");
    }

    #[test]
    fn test_parse_hci_acl_recv() {
        let line = "[00:00:01.139] ACL <= 80 20 0a 00 06 00 01 00 0a 02 02 00 02 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_ts, flags, data) = result.unwrap();
        assert_eq!(flags, 3);
        assert_eq!(data[0], 0x02, "ACL should have H4 type 0x02");
    }

    #[test]
    fn test_parse_with_pc_ts() {
        let line = "(15:20:52.807) [00:00:19.017] ACL <= 80 20 08 00 04 00 42 00 01 53 01 9c";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_ts, flags, data) = result.unwrap();
        assert_eq!(flags, 3);
        assert_eq!(data[0], 0x02, "ACL should have H4 type 0x02");
    }

    #[test]
    fn test_skip_msg() {
        let line = "[00:00:00.102] MSG <- 60 01 01";
        assert!(parse_hci_line(line).is_none());
    }

    #[test]
    fn test_skip_msg_with_pc_ts() {
        let line = "(15:20:53.209) [00:00:19.343] MSG <- 65 01 01";
        assert!(parse_hci_line(line).is_none());
    }

    #[test]
    fn test_skip_non_hci() {
        let line = "Hello Platform";
        assert!(parse_hci_line(line).is_none());
    }

    #[test]
    fn test_hex_parse() {
        let bytes = parse_hex_bytes("03 0c 00").unwrap();
        assert_eq!(bytes, vec![0x03, 0x0c, 0x00]);
    }

    #[test]
    fn test_btsnoop_epoch_offset() {
        let offset = btsnoop_epoch_offset_usec();
        // Roughly 1970 years in microseconds
        assert!(offset > 62_000_000_000_000_000);
        assert!(offset < 63_000_000_000_000_000);
    }
}
