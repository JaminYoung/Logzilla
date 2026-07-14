use std::io::Write;

struct HciPacket {
    timestamp_usec: u64,
    flags: u32,
    data: Vec<u8>,
}

/// Wall-clock time of day with millisecond precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimeOfDay {
    h: u32,
    m: u32,
    s: u32,
    ms: u32,
}

impl TimeOfDay {
    fn total_ms(self) -> i64 {
        ((self.h as i64) * 3600 + (self.m as i64) * 60 + (self.s as i64)) * 1000 + self.ms as i64
    }

    fn from_total_ms(total: i64) -> Self {
        // Keep within a single day for wall-clock display; wrap if needed.
        let day = 24 * 3600 * 1000;
        let mut t = total % day;
        if t < 0 {
            t += day;
        }
        let ms = (t % 1000) as u32;
        let sec_total = t / 1000;
        let s = (sec_total % 60) as u32;
        let min_total = sec_total / 60;
        let m = (min_total % 60) as u32;
        let h = (min_total / 60) as u32;
        Self { h, m, s, ms }
    }
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

fn clamp_tz_hours(tz_hours: i32) -> i32 {
    tz_hours.clamp(-12, 14)
}

/// Encode a log wall-clock time of day into btsnoop absolute microseconds.
///
/// `log_tz_hours` is the timezone of the **computer log** timestamps
/// (setting “日志时区”, default +8). The wall-clock is interpreted as local
/// time in that fixed offset, then converted to true UTC for btsnoop storage.
///
/// Viewer implications (same absolute instant):
/// - Frontline showing UTC+8: wall clock matches log when log_tz = +8
/// - WPS showing UTC0: shows UTC (= wall − log_tz). Set 日志时区 to 0 if you
///   want WPS to display the same H:M:S digits as the log (no conversion).
///
/// Y/M/D uses “today” in the log timezone; only H:M:S.mmm must be accurate.
fn wall_clock_to_btsnoop(tod: TimeOfDay, log_tz_hours: i32) -> u64 {
    let log_tz_hours = clamp_tz_hours(log_tz_hours);
    let offset_secs = log_tz_hours * 3600;
    let offset = chrono::FixedOffset::east_opt(offset_secs)
        .or_else(|| chrono::FixedOffset::east_opt(0))
        .unwrap();

    // Calendar date from "today" in the log timezone (Y/M/D not critical).
    let date = chrono::Utc::now().with_timezone(&offset).date_naive();
    let naive = chrono::NaiveDateTime::new(
        date,
        chrono::NaiveTime::from_hms_milli_opt(tod.h, tod.m, tod.s, tod.ms)
            .unwrap_or_else(|| chrono::NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap()),
    );

    // Interpret wall clock as local time in log timezone → absolute UTC.
    let dt = naive
        .and_local_timezone(offset)
        .single()
        .or_else(|| naive.and_local_timezone(offset).earliest())
        .unwrap_or_else(|| naive.and_utc().fixed_offset());
    let unix_usec = dt.timestamp_micros() as u64;
    unix_usec_to_btsnoop(unix_usec)
}

fn write_btsnoop<W: Write>(writer: &mut W, packets: &[HciPacket]) -> Result<(), String> {
    writer.write_all(b"btsnoop\0").map_err(|e| e.to_string())?;
    writer.write_all(&1u32.to_be_bytes()).map_err(|e| e.to_string())?;
    // datalink = 1002 (HCI UART / H4) — Frontline expects 1002 for H4
    writer.write_all(&1002u32.to_be_bytes()).map_err(|e| e.to_string())?;

    for pkt in packets {
        let packet_len = pkt.data.len() as u32;

        writer
            .write_all(&packet_len.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&packet_len.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&pkt.flags.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&0u32.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&pkt.timestamp_usec.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer.write_all(&pkt.data).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Parse optional PC `(HH:MM:SS.mmm)` at the start of a line.
/// Returns (pc_ts, remainder after PC timestamp).
fn parse_pc_timestamp_prefix(line: &str) -> (Option<TimeOfDay>, &str) {
    let line = line.trim_start();
    if !line.starts_with('(') {
        return (None, line);
    }
    let Some(close_paren) = line.find(')') else {
        return (None, line);
    };
    let ts_str = &line[1..close_paren];
    if ts_str.len() != 12 || !ts_str.is_ascii() {
        return (None, line);
    }
    // Require colon/dot separators: HH:MM:SS.mmm
    if ts_str.as_bytes()[2] != b':'
        || ts_str.as_bytes()[5] != b':'
        || ts_str.as_bytes()[8] != b'.'
    {
        return (None, line);
    }
    let Ok(h) = ts_str[0..2].parse::<u32>() else {
        return (None, line);
    };
    let Ok(m) = ts_str[3..5].parse::<u32>() else {
        return (None, line);
    };
    let Ok(s) = ts_str[6..8].parse::<u32>() else {
        return (None, line);
    };
    let Ok(ms) = ts_str[9..12].parse::<u32>() else {
        return (None, line);
    };
    if h > 23 || m > 59 || s > 59 || ms > 999 {
        return (None, line);
    }
    (
        Some(TimeOfDay { h, m, s, ms }),
        line[close_paren + 1..].trim_start(),
    )
}

/// Parse optional chip `[HH:MM:SS.mmm]` at the start of remainder.
fn parse_chip_timestamp_prefix(remaining: &str) -> (Option<TimeOfDay>, &str) {
    let remaining = remaining.trim_start();
    if !remaining.starts_with('[') {
        return (None, remaining);
    }
    let Some(close_bracket) = remaining.find(']') else {
        return (None, remaining);
    };
    let ts_part = &remaining[1..close_bracket];
    if ts_part.len() != 12 || !ts_part.is_ascii() {
        return (None, remaining);
    }
    if ts_part.as_bytes()[2] != b':'
        || ts_part.as_bytes()[5] != b':'
        || ts_part.as_bytes()[8] != b'.'
    {
        return (None, remaining);
    }
    let Ok(h) = ts_part[0..2].parse::<u32>() else {
        return (None, remaining);
    };
    let Ok(m) = ts_part[3..5].parse::<u32>() else {
        return (None, remaining);
    };
    let Ok(s) = ts_part[6..8].parse::<u32>() else {
        return (None, remaining);
    };
    let Ok(ms) = ts_part[9..12].parse::<u32>() else {
        return (None, remaining);
    };
    if h > 23 || m > 59 || s > 59 || ms > 999 {
        return (None, remaining);
    }
    (
        Some(TimeOfDay { h, m, s, ms }),
        remaining[close_bracket + 1..].trim_start(),
    )
}

/// Parse a single HCI packet line into (pc, chip, flags, h4_data).
///
/// Accepts:
///   (15:20:52.807) [00:00:19.017] CMD => 03 0c 00
///   [00:00:19.017] CMD => 03 0c 00
///   CMD => 03 0c 00
///
/// H4 mapping: CMD → 0x01, ACL → 0x02, EVT → 0x04
fn parse_hci_line(
    line: &str,
) -> Option<(Option<TimeOfDay>, Option<TimeOfDay>, u32, Vec<u8>)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (pc_ts, remaining) = parse_pc_timestamp_prefix(line);
    let (chip_ts, rest) = parse_chip_timestamp_prefix(remaining);
    if rest.len() < 6 {
        return None;
    }

    let space_after_type = rest.find(' ')?;
    let pkt_type = &rest[..space_after_type];
    let after_type = rest[space_after_type + 1..].trim_start();
    if after_type.len() < 2 {
        return None;
    }
    let dir_str = &after_type[..2];
    let direction_sent = match dir_str {
        "=>" => true,
        "<=" => false,
        _ => return None,
    };

    let hex_part = after_type[2..].trim();
    if hex_part.is_empty() {
        return None;
    }
    let raw_bytes = parse_hex_bytes(hex_part).ok()?;
    if raw_bytes.is_empty() {
        return None;
    }

    let (flags, h4_type): (u32, u8) = match (pkt_type, direction_sent) {
        ("CMD", true) => (0, 0x01),
        ("EVT", false) => (1, 0x04),
        ("ACL", true) => (2, 0x02),
        ("ACL", false) => (3, 0x02),
        _ => return None,
    };

    let mut data_with_h4 = Vec::with_capacity(1 + raw_bytes.len());
    data_with_h4.push(h4_type);
    data_with_h4.extend_from_slice(&raw_bytes);

    Some((pc_ts, chip_ts, flags, data_with_h4))
}

/// Track last seen PC wall-clock and optional chip anchor for delta carry.
struct TimestampState {
    /// Last PC wall-clock time of day (from any line, including non-HCI).
    last_pc: Option<TimeOfDay>,
    /// Chip time observed together with `last_pc` (for relative advance).
    last_pc_chip: Option<TimeOfDay>,
    /// Last resolved wall-clock used for an HCI packet.
    last_resolved: Option<TimeOfDay>,
    /// Chip time at `last_resolved` (for pure chip-only sequences).
    last_resolved_chip: Option<TimeOfDay>,
}

impl TimestampState {
    fn new() -> Self {
        Self {
            last_pc: None,
            last_pc_chip: None,
            last_resolved: None,
            last_resolved_chip: None,
        }
    }

    /// Observe a PC timestamp from any log line (HCI or non-HCI).
    fn observe_pc(&mut self, pc: TimeOfDay, chip: Option<TimeOfDay>) {
        self.last_pc = Some(pc);
        if let Some(c) = chip {
            self.last_pc_chip = Some(c);
        }
        // A fresh PC reading is also a valid resolved base.
        self.last_resolved = Some(pc);
        if let Some(c) = chip {
            self.last_resolved_chip = Some(c);
        }
    }

    /// Resolve wall-clock for one HCI packet.
    ///
    /// Priority:
    /// 1. PC on this HCI line
    /// 2. Last PC from any prior line, advanced by chip delta when possible
    /// 3. Advance last resolved HCI time by chip delta
    /// 4. Chip wall-clock as last resort (only before any PC is seen)
    fn resolve_hci(
        &mut self,
        pc: Option<TimeOfDay>,
        chip: Option<TimeOfDay>,
    ) -> Option<TimeOfDay> {
        if let Some(pc_tod) = pc {
            self.observe_pc(pc_tod, chip);
            return Some(pc_tod);
        }

        // No PC on this HCI line: prefer last PC (+ chip delta).
        if let Some(base_pc) = self.last_pc {
            let resolved = match (chip, self.last_pc_chip) {
                (Some(cur_chip), Some(base_chip)) => {
                    let delta = cur_chip.total_ms() - base_chip.total_ms();
                    TimeOfDay::from_total_ms(base_pc.total_ms() + delta)
                }
                _ => base_pc,
            };
            self.last_resolved = Some(resolved);
            if let Some(c) = chip {
                self.last_resolved_chip = Some(c);
                // Keep chip anchor in sync so subsequent deltas stay continuous.
                // Do not overwrite last_pc; only update chip anchor relative to last_pc.
                if self.last_pc_chip.is_none() {
                    self.last_pc_chip = Some(c);
                }
            }
            return Some(resolved);
        }

        // Never saw any PC: fall back to chip progression / absolute chip.
        if let Some(cur_chip) = chip {
            let resolved = if let (Some(prev_res), Some(prev_chip)) =
                (self.last_resolved, self.last_resolved_chip)
            {
                let delta = cur_chip.total_ms() - prev_chip.total_ms();
                TimeOfDay::from_total_ms(prev_res.total_ms() + delta)
            } else {
                // Absolute chip wall-clock (rare bootstrap case).
                cur_chip
            };
            self.last_resolved = Some(resolved);
            self.last_resolved_chip = Some(cur_chip);
            return Some(resolved);
        }

        self.last_resolved
    }
}

#[tauri::command]
pub fn extract_hci(
    input_path: String,
    timezone_offset_hours: Option<i32>,
) -> Result<String, String> {
    // 日志时区：电脑日志墙钟所属时区（默认 UTC+8），导出时转成真 UTC 写入 btsnoop。
    // 默认 0：按 UTC0 写入墙钟数字，WPS 显示与日志一致；Frontline(UTC+8) 用户可改为 +8。
    let tz_hours = clamp_tz_hours(timezone_offset_hours.unwrap_or(0));

    let bytes =
        std::fs::read(&input_path).map_err(|e| format!("无法读取文件: {}", e))?;
    let content = String::from_utf8_lossy(&bytes);

    let mut packets: Vec<HciPacket> = Vec::new();
    let mut state = TimestampState::new();

    for line in content.lines() {
        // Always harvest PC timestamps from non-HCI lines too
        // e.g. "(17:20:54.438) [00:00:03.404] MSG <- 60 01 01"
        // so the next chip-only HCI line can inherit the PC wall-clock.
        let (line_pc, after_pc) = parse_pc_timestamp_prefix(line);
        let (line_chip, _) = parse_chip_timestamp_prefix(after_pc);
        if let Some(pc) = line_pc {
            // Only update state here for non-HCI lines; HCI path also updates.
            // For HCI lines parse_hci_line succeeds and resolve_hci handles it.
            if parse_hci_line(line).is_none() {
                state.observe_pc(pc, line_chip);
            }
        }

        if let Some((pc_raw, chip_raw, flags, data)) = parse_hci_line(line) {
            let Some(tod) = state.resolve_hci(pc_raw, chip_raw) else {
                continue;
            };
            let ts = wall_clock_to_btsnoop(tod, tz_hours);
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

    let file =
        std::fs::File::create(&output_path).map_err(|e| format!("无法创建输出文件: {}", e))?;
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
    use chrono::Timelike;
    use std::path::PathBuf;

    fn tod(h: u32, m: u32, s: u32, ms: u32) -> TimeOfDay {
        TimeOfDay { h, m, s, ms }
    }

    #[test]
    fn test_extract_from_sample_log() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sample_path = manifest_dir.join("..").join("COM6_2026-05-23_15-20-33.txt");
        if !sample_path.exists() {
            eprintln!("Sample log not found at {:?}, skipping", sample_path);
            return;
        }
        let result = extract_hci(sample_path.to_str().unwrap().to_string(), Some(8));
        assert!(result.is_ok(), "extract_hci failed: {:?}", result.err());
        let msg = result.unwrap();
        assert!(msg.contains("个 HCI 数据包"), "Unexpected result: {}", msg);
        let cfa_path = sample_path.with_extension("cfa");
        assert!(cfa_path.exists(), "CFA file not created at {:?}", cfa_path);
        let data = std::fs::read(&cfa_path).expect("Failed to read CFA");
        assert!(data.len() > 16, "CFA file too small");
        assert_eq!(&data[0..8], b"btsnoop\0", "Invalid btsnoop magic");
        let mut ver = [0u8; 4];
        ver.copy_from_slice(&data[8..12]);
        assert_eq!(u32::from_be_bytes(ver), 1, "Wrong btsnoop version");
        let mut dl = [0u8; 4];
        dl.copy_from_slice(&data[12..16]);
        assert_eq!(u32::from_be_bytes(dl), 1002, "Expected datalink 1002 (H4)");
        let _ = std::fs::remove_file(&cfa_path);
    }

    #[test]
    fn test_parse_hci_cmd() {
        let line = "[00:00:00.107] CMD => 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (pc, chip, flags, data) = result.unwrap();
        assert!(pc.is_none());
        assert_eq!(chip, Some(tod(0, 0, 0, 107)));
        assert_eq!(flags, 0);
        assert_eq!(&data, &[0x01, 0x03, 0x0c, 0x00]);
    }

    #[test]
    fn test_parse_hci_evt() {
        let line = "[00:00:00.128] EVT <= 0e 04 05 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_pc, _chip, flags, data) = result.unwrap();
        assert_eq!(flags, 1);
        assert_eq!(data[0], 0x04, "EVT should have H4 type 0x04");
    }

    #[test]
    fn test_parse_hci_acl_sent() {
        let line =
            "[00:00:01.140] ACL => 80 00 10 00 0c 00 01 00 0b 02 08 00 02 00 00 00 80 02 00 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_pc, _chip, flags, data) = result.unwrap();
        assert_eq!(flags, 2);
        assert_eq!(data[0], 0x02, "ACL should have H4 type 0x02");
    }

    #[test]
    fn test_parse_hci_acl_recv() {
        let line = "[00:00:01.139] ACL <= 80 20 0a 00 06 00 01 00 0a 02 02 00 02 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (_pc, _chip, flags, data) = result.unwrap();
        assert_eq!(flags, 3);
        assert_eq!(data[0], 0x02, "ACL should have H4 type 0x02");
    }

    #[test]
    fn test_parse_with_pc_ts() {
        let line =
            "(15:20:52.807) [00:00:19.017] ACL <= 80 20 08 00 04 00 42 00 01 53 01 9c";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (pc, chip, flags, data) = result.unwrap();
        assert_eq!(pc, Some(tod(15, 20, 52, 807)));
        assert_eq!(chip, Some(tod(0, 0, 19, 17)));
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
        assert!(offset > 62_000_000_000_000_000);
        assert!(offset < 63_000_000_000_000_000);
    }

    #[test]
    fn test_parse_no_ts_packet() {
        let line = "CMD => 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (pc, chip, flags, data) = result.unwrap();
        assert!(pc.is_none());
        assert!(chip.is_none());
        assert_eq!(flags, 0);
        assert_eq!(&data, &[0x01, 0x03, 0x0c, 0x00]);
    }

    #[test]
    fn test_parse_pc_ts_only() {
        let line = "(15:20:52.807) CMD => 03 0c 00";
        let result = parse_hci_line(line);
        assert!(result.is_some());
        let (pc, chip, flags, data) = result.unwrap();
        assert_eq!(pc, Some(tod(15, 20, 52, 807)));
        assert!(chip.is_none());
        assert_eq!(flags, 0);
        assert_eq!(&data, &[0x01, 0x03, 0x0c, 0x00]);
    }

    #[test]
    fn test_extract_carry_forward() {
        let dir = std::env::temp_dir();
        let path = dir.join("logzilla_carry_fwd_test.txt");
        let content = "[00:00:00.107] CMD => 03 0c 00\nCMD => 04 0c 00\n";
        std::fs::write(&path, content).unwrap();
        let result = extract_hci(path.to_str().unwrap().to_string(), Some(8));
        let _ = std::fs::remove_file(path.with_extension("cfa"));
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok(), "extract_hci failed: {:?}", result.err());
        let msg = result.unwrap();
        assert!(
            msg.contains("2 个 HCI 数据包"),
            "expected 2 packets (carry-forward), got: {}",
            msg
        );
    }

    #[test]
    fn test_extract_gbk_lossy() {
        let dir = std::env::temp_dir();
        let path = dir.join("logzilla_gbk_lossy_test.txt");
        let mut bytes: Vec<u8> = vec![0x81, 0x40, 0x0A];
        bytes.extend_from_slice(b"[00:00:00.107] CMD => 03 0c 00\n");
        std::fs::write(&path, &bytes).unwrap();
        let result = extract_hci(path.to_str().unwrap().to_string(), Some(8));
        let _ = std::fs::remove_file(path.with_extension("cfa"));
        let _ = std::fs::remove_file(&path);
        assert!(
            result.is_ok(),
            "GBK-ish file should not fail read: {:?}",
            result.err()
        );
        let msg = result.unwrap();
        assert!(
            msg.contains("1 个 HCI 数据包"),
            "expected 1 packet from lossy-decoded GBK file, got: {}",
            msg
        );
    }

    #[test]
    fn test_resolve_uses_non_hci_pc_then_chip_delta() {
        // Real log shape from 20260713-172213-395.txt:
        // (17:20:54.438) [00:00:03.404] MSG ...   ← not HCI, but has PC
        // [00:00:03.411] CMD => 03 0c 00           ← HCI reset, chip-only
        // Expect wall-clock ≈ 17:20:54.445 (PC + 7ms chip delta)
        let mut state = TimestampState::new();
        state.observe_pc(tod(17, 20, 54, 438), Some(tod(0, 0, 3, 404)));

        let reset = state
            .resolve_hci(None, Some(tod(0, 0, 3, 411)))
            .expect("should resolve");
        assert_eq!(reset, tod(17, 20, 54, 445));

        // Next chip-only line continues relative to same PC anchor.
        let next = state
            .resolve_hci(None, Some(tod(0, 0, 3, 433)))
            .expect("should resolve");
        assert_eq!(next, tod(17, 20, 54, 467));
    }

    #[test]
    fn test_resolve_prefers_hci_pc_over_chip() {
        let mut state = TimestampState::new();
        let t = state
            .resolve_hci(Some(tod(15, 20, 52, 807)), Some(tod(0, 0, 19, 17)))
            .unwrap();
        assert_eq!(t, tod(15, 20, 52, 807));

        // Chip-only after PC: advance by chip delta from anchor.
        let t2 = state
            .resolve_hci(None, Some(tod(0, 0, 19, 50)))
            .unwrap();
        // base PC 15:20:52.807 at chip 19.017; chip 19.050 → +33ms
        assert_eq!(t2, tod(15, 20, 52, 840));
    }

    #[test]
    fn test_wall_clock_converts_log_tz_to_true_utc() {
        // Log wall clock 17:20:54.438 in UTC+8 must store absolute UTC 09:20:54.438
        // so Frontline (UTC+8 viewer) shows 17:20:54 again.
        let ts = wall_clock_to_btsnoop(tod(17, 20, 54, 438), 8);
        let unix_usec = ts - btsnoop_epoch_offset_usec();
        let dt = chrono::DateTime::from_timestamp_micros(unix_usec as i64)
            .expect("valid unix usec");
        let utc = dt.naive_utc();
        assert_eq!(utc.time().hour(), 9);
        assert_eq!(utc.time().minute(), 20);
        assert_eq!(utc.time().second(), 54);
        assert_eq!(utc.time().nanosecond() / 1_000_000, 438);

        // log_tz=0: preserve digits for WPS UTC0 viewers.
        let ts0 = wall_clock_to_btsnoop(tod(17, 20, 54, 438), 0);
        let unix0 = ts0 - btsnoop_epoch_offset_usec();
        let n0 = chrono::DateTime::from_timestamp_micros(unix0 as i64)
            .unwrap()
            .naive_utc();
        assert_eq!(n0.time().hour(), 17);
        assert_eq!(n0.time().minute(), 20);
        assert_eq!(n0.time().second(), 54);
        assert_eq!(n0.time().nanosecond() / 1_000_000, 438);

        // Delta between tz0 and tz8 for same wall clock is exactly 8 hours.
        assert_eq!(
            (ts0 as i64) - (ts as i64),
            8 * 3600 * 1_000_000
        );
    }

    #[test]
    fn test_extract_from_user_scenario_non_hci_pc() {
        let dir = std::env::temp_dir();
        let path = dir.join("logzilla_user_scenario_pc_msg.txt");
        let content = "\
(17:20:54.388) sys_init  dcin :0
(17:20:54.438) [00:00:03.404] MSG <- 60 01 01
[00:00:03.411] CMD => 03 0c 00
[00:00:03.433] EVT <= 0e 04 05 03 0c 00
(17:20:55.099) [00:00:04.066] CMD => 17 20 20 01 02 03 04
";
        std::fs::write(&path, content).unwrap();
        let result = extract_hci(path.to_str().unwrap().to_string(), Some(8));
        assert!(result.is_ok(), "extract_hci failed: {:?}", result.err());

        let cfa_path = path.with_extension("cfa");
        let data = std::fs::read(&cfa_path).expect("Failed to read CFA");
        let _ = std::fs::remove_file(&cfa_path);
        let _ = std::fs::remove_file(&path);

        assert!(data.len() > 16);
        let mut ts0 = [0u8; 8];
        ts0.copy_from_slice(&data[32..40]);
        let t0 = u64::from_be_bytes(ts0) - btsnoop_epoch_offset_usec();
        let dt0 = chrono::DateTime::from_timestamp_micros(t0 as i64).unwrap();
        let n0 = dt0.naive_utc();
        // Wall 17:20:54.445 @ UTC+8 → stored UTC 09:20:54.445
        assert_eq!(n0.time().hour(), 9);
        assert_eq!(n0.time().minute(), 20);
        assert_eq!(n0.time().second(), 54);
        assert_eq!(n0.time().nanosecond() / 1_000_000, 445);

        // Third packet PC 17:20:55.099 @ UTC+8 → UTC 09:20:55.099
        let mut ts2 = [0u8; 8];
        ts2.copy_from_slice(&data[91..99]);
        let t2 = u64::from_be_bytes(ts2) - btsnoop_epoch_offset_usec();
        let dt2 = chrono::DateTime::from_timestamp_micros(t2 as i64).unwrap();
        let n2 = dt2.naive_utc();
        assert_eq!(n2.time().hour(), 9);
        assert_eq!(n2.time().minute(), 20);
        assert_eq!(n2.time().second(), 55);
        assert_eq!(n2.time().nanosecond() / 1_000_000, 99);
    }

    #[test]
    fn test_extract_real_user_file_if_present() {
        let path = PathBuf::from(r"D:\log\07_13\bt_t1t2\20260713-172213-395.txt");
        if !path.exists() {
            eprintln!("user sample not present, skip");
            return;
        }
        let result = extract_hci(path.to_str().unwrap().to_string(), Some(8));
        assert!(result.is_ok(), "extract_hci failed: {:?}", result.err());
        let cfa = path.with_extension("cfa");
        let data = std::fs::read(&cfa).expect("read cfa");
        let mut ts0 = [0u8; 8];
        ts0.copy_from_slice(&data[32..40]);
        let t0 = u64::from_be_bytes(ts0) - btsnoop_epoch_offset_usec();
        let n0 = chrono::DateTime::from_timestamp_micros(t0 as i64)
            .unwrap()
            .naive_utc();
        // Wall ~17:20:54.xxx @ UTC+8 → stored UTC ~09:20:54
        assert_eq!(
            n0.time().hour(),
            9,
            "first HCI must be true UTC of PC wall clock, got {:?}",
            n0
        );
        assert_eq!(n0.time().minute(), 20);
        assert_eq!(n0.time().second(), 54);
        let _ = std::fs::remove_file(&cfa);
    }
}
