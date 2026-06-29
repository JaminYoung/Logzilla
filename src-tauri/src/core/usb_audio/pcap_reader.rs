use std::fs;

pub struct PcapPacket {
    pub timestamp: f64,
    pub data: Vec<u8>,
}

pub fn read_pcap(path: &str) -> Result<Vec<PcapPacket>, String> {
    let data = fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    if data.len() < 4 {
        return Err("文件太小，不是有效的 pcap/pcapng".to_string());
    }
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    match magic {
        0xa1b2c3d4 | 0xd4c3b2a1 => read_classic_pcap(&data, magic),
        0x0a0d0d0a => read_pcapng(&data),
        _ => Err(format!("不支持的文件格式 (magic: 0x{:08X})", magic)),
    }
}

fn read_classic_pcap(data: &[u8], magic: u32) -> Result<Vec<PcapPacket>, String> {
    let le = magic == 0xa1b2c3d4;
    let mut offset = 24;
    let mut packets = Vec::new();

    while offset + 16 <= data.len() {
        let ts_sec = r32(data, offset, le); offset += 4;
        let ts_usec = r32(data, offset, le); offset += 4;
        let incl_len = r32(data, offset, le) as usize; offset += 4;
        let _orig_len = r32(data, offset, le) as usize; offset += 4;

        if offset + incl_len > data.len() { break; }
        let packet_data = data[offset..offset + incl_len].to_vec();
        offset += incl_len;

        packets.push(PcapPacket {
            timestamp: ts_sec as f64 + (ts_usec as f64) / 1_000_000.0,
            data: packet_data,
        });
    }
    Ok(packets)
}

fn read_pcapng(data: &[u8]) -> Result<Vec<PcapPacket>, String> {
    let mut offset = 0;
    let mut packets: Vec<PcapPacket> = Vec::new();
    let mut iface_resol: f64 = 1e-6;

    while offset + 12 <= data.len() {
        let block_type = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        let block_len = u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]) as usize;

        if block_len < 12 || offset + block_len > data.len() { break; }

        match block_type {
            0x00000001 => {
                if offset + 16 <= data.len() {
                    let opt_offset = offset + 16;
                    let opt_end = offset + block_len - 4;
                    if let Some(resol) = read_pcapng_option(&data[opt_offset..opt_end], 9) {
                        if resol & 0x80 != 0 {
                            iface_resol = 2f64.powi(-((resol & 0x7F) as i32));
                        } else {
                            iface_resol = 10f64.powi(-(resol as i32));
                        }
                    }
                }
            }
            0x00000003 => {
                if offset + 32 <= data.len() {
                    let orig_len = u32::from_le_bytes([data[offset+20], data[offset+21], data[offset+22], data[offset+23]]) as usize;
                    let pkt_data = data[offset+24..offset+24+orig_len].to_vec();
                    let ts = if !packets.is_empty() {
                        packets.last().unwrap().timestamp + 1e-6
                    } else { 0.0 };
                    packets.push(PcapPacket { timestamp: ts, data: pkt_data });
                }
            }
            0x00000006 => {
                if offset + 28 <= data.len() {
                    let ts_hi = u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]);
                    let ts_lo = u32::from_le_bytes([data[offset+16], data[offset+17], data[offset+18], data[offset+19]]);
                    let cap_len = u32::from_le_bytes([data[offset+20], data[offset+21], data[offset+22], data[offset+23]]) as usize;
                    let ts_raw = ((ts_hi as u64) << 32) | (ts_lo as u64);
                    if offset + 28 + cap_len <= data.len() {
                        let pkt_data = data[offset+28..offset+28+cap_len].to_vec();
                        packets.push(PcapPacket {
                            timestamp: (ts_raw as f64) * iface_resol,
                            data: pkt_data,
                        });
                    }
                }
            }
            _ => {}
        }
        offset += block_len;
    }
    Ok(packets)
}

fn read_pcapng_option(opts: &[u8], target_code: u16) -> Option<u32> {
    let mut o = 0;
    while o + 4 <= opts.len() {
        let code = u16::from_le_bytes([opts[o], opts[o+1]]);
        let len = u16::from_le_bytes([opts[o+2], opts[o+3]]) as usize;
        o += 4;
        if code == 0 { break; }
        if code == target_code && len == 1 && o < opts.len() {
            return Some(opts[o] as u32);
        }
        if code == target_code && len == 4 && o + 4 <= opts.len() {
            return Some(u32::from_le_bytes([opts[o], opts[o+1], opts[o+2], opts[o+3]]));
        }
        o += len;
        o = (o + 3) & !3;
    }
    None
}

fn r32(buf: &[u8], off: usize, le: bool) -> u32 {
    let b = &buf[off..off+4];
    if le { u32::from_le_bytes([b[0],b[1],b[2],b[3]]) }
    else { u32::from_be_bytes([b[0],b[1],b[2],b[3]]) }
}
