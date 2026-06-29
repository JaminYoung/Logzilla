use std::collections::HashMap;
use std::path::Path;
use super::pcap_reader;
use super::usbpcap::{self, Direction, IsoPacket, SetInterfaceEvent};
use super::descriptors;
use super::export;

#[derive(Debug, Clone)]
pub struct Segment {
    pub packets: Vec<IsoPacket>,
    pub interface: u8,
    pub direction: Direction,
    pub start_ts: f64,
    pub audio_params: Option<descriptors::AudioParams>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitMode { A, B, C }

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub interface: u8,
    pub endpoint: u8,
    pub direction: String,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub channels: u8,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub file_name: String,
    pub total_packets: usize,
    pub iso_packets: usize,
    pub interfaces: Vec<InterfaceInfo>,
    pub set_interface_count: usize,
}

#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub segment_count: usize,
    pub files: Vec<String>,
}

pub fn analyze(path: &str) -> Result<AnalysisResult, String> {
    let raw_packets = pcap_reader::read_pcap(path)?;
    let total = raw_packets.len();

    let mut iso_count: usize = 0;
    let mut set_iface_events: Vec<SetInterfaceEvent> = Vec::new();
    let mut descriptor_blobs: Vec<Vec<u8>> = Vec::new();

    for pkt in &raw_packets {
        match usbpcap::classify(&pkt.data, pkt.timestamp) {
            usbpcap::Classified::Iso(_) => iso_count += 1,
            usbpcap::Classified::SetInterface(evt) => set_iface_events.push(evt),
            usbpcap::Classified::DescriptorResponse(data) => descriptor_blobs.push(data),
            usbpcap::Classified::Other => {}
        }
    }

    let ep_info = descriptors::parse_audio_descriptors(&descriptor_blobs);

    let mut interfaces: Vec<InterfaceInfo> = Vec::new();
    let mut seen: Vec<(u8, u8)> = Vec::new();

    for (ep, info) in &ep_info {
        if seen.contains(&(info.interface, *ep)) { continue; }
        seen.push((info.interface, *ep));
        let dir_str = if ep & 0x80 != 0 { "in" } else { "out" };
        let params = info.params.as_ref();
        interfaces.push(InterfaceInfo {
            interface: info.interface,
            endpoint: *ep,
            direction: dir_str.to_string(),
            sample_rate: params.map(|p| p.sample_rate).unwrap_or(0),
            bit_depth: params.map(|p| p.bit_depth).unwrap_or(0),
            channels: params.map(|p| p.channels).unwrap_or(0),
        });
    }

    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(AnalysisResult {
        file_name,
        total_packets: total,
        iso_packets: iso_count,
        interfaces,
        set_interface_count: set_iface_events.len(),
    })
}

pub fn extract(
    path: &str,
    mode: SplitMode,
    threshold_ms: u64,
    output_dir: &str,
) -> Result<ExtractResult, String> {
    let raw_packets = pcap_reader::read_pcap(path)?;

    let mut iso_packets: Vec<IsoPacket> = Vec::new();
    let mut set_iface_events: Vec<SetInterfaceEvent> = Vec::new();
    let mut descriptor_blobs: Vec<Vec<u8>> = Vec::new();

    for pkt in &raw_packets {
        match usbpcap::classify(&pkt.data, pkt.timestamp) {
            usbpcap::Classified::Iso(iso) => iso_packets.push(iso),
            usbpcap::Classified::SetInterface(evt) => set_iface_events.push(evt),
            usbpcap::Classified::DescriptorResponse(data) => descriptor_blobs.push(data),
            _ => {}
        }
    }

    let ep_info = descriptors::parse_audio_descriptors(&descriptor_blobs);

    let mut ep_to_iface: HashMap<u8, u8> = HashMap::new();
    let mut ep_to_params: HashMap<u8, descriptors::AudioParams> = HashMap::new();
    for (ep, info) in &ep_info {
        ep_to_iface.insert(*ep, info.interface);
        if let Some(ref params) = info.params {
            ep_to_params.insert(*ep, params.clone());
        }
    }

    for pkt in iso_packets.iter_mut() {
        if let Some(&iface) = ep_to_iface.get(&pkt.endpoint) {
            pkt.interface = iface;
        }
    }

    iso_packets.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal));
    set_iface_events.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal));

    let segments = match mode {
        SplitMode::A => build_segments_a(&iso_packets, &set_iface_events, threshold_ms),
        SplitMode::B => build_segments_b(&iso_packets, &set_iface_events, threshold_ms),
        SplitMode::C => build_segments_c(&iso_packets, &ep_to_params),
    };

    if !Path::new(output_dir).exists() {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    let mut saved_files: Vec<String> = Vec::new();
    for seg in &segments {
        let dir_str = match seg.direction { Direction::In => "in", Direction::Out => "out" };
        let dir_arrow = match seg.direction { Direction::In => "in", Direction::Out => "out" };
        let ts = format_timestamp(seg.start_ts);
        let base = format!("{}_inf{}_{}", dir_arrow, seg.interface, dir_str);
        let base_full = format!("{}_{}", ts, base);

        let default_params = descriptors::AudioParams {
            sample_rate: 48000, bit_depth: 16,
            channels: if seg.direction == Direction::In { 1 } else { 2 },
        };
        let params = seg.audio_params.as_ref().unwrap_or(&default_params);

        let total_data: Vec<u8> = seg.packets.iter().flat_map(|p| p.data.iter().copied()).collect();
        if total_data.is_empty() { continue; }

        let wav_path = format!("{}\\{}.wav", output_dir, base_full);
        export::save_wav(&total_data, &wav_path, params.sample_rate, params.channels as u16, params.bit_depth as u16)?;
        saved_files.push(wav_path);

        let raw_path = format!("{}\\{}.raw", output_dir, base_full);
        export::save_raw(&total_data, &raw_path)?;
        saved_files.push(raw_path);
    }

    Ok(ExtractResult {
        segment_count: segments.len(),
        files: saved_files,
    })
}

fn build_segments_a(
    iso: &[IsoPacket],
    set_iface: &[SetInterfaceEvent],
    threshold_ms: u64,
) -> Vec<Segment> {
    let mut groups: HashMap<(u8, Direction), Vec<&IsoPacket>> = HashMap::new();
    for pkt in iso {
        groups.entry((pkt.interface, pkt.direction)).or_default().push(pkt);
    }

    let mut segments = Vec::new();

    for ((iface, dir), pkts) in &groups {
        let relevant_splits: Vec<f64> = set_iface.iter()
            .filter(|e| e.interface == *iface)
            .map(|e| e.timestamp)
            .collect();

        let segs = split_group(pkts, &relevant_splits, threshold_ms, *iface, *dir);
        segments.extend(segs);
    }

    segments
}

fn build_segments_b(
    iso: &[IsoPacket],
    set_iface: &[SetInterfaceEvent],
    threshold_ms: u64,
) -> Vec<Segment> {
    let mut groups: HashMap<Direction, Vec<&IsoPacket>> = HashMap::new();
    for pkt in iso {
        groups.entry(pkt.direction).or_default().push(pkt);
    }

    let all_splits: Vec<f64> = set_iface.iter().map(|e| e.timestamp).collect();
    let mut segments = Vec::new();

    for (dir, pkts) in &groups {
        let first_iface = pkts.first().map(|p| p.interface).unwrap_or(0);
        let segs = split_group(pkts, &all_splits, threshold_ms, first_iface, *dir);
        segments.extend(segs);
    }

    segments
}

fn build_segments_c(
    iso: &[IsoPacket],
    ep_to_params: &HashMap<u8, descriptors::AudioParams>,
) -> Vec<Segment> {
    let mut groups: HashMap<Direction, Vec<&IsoPacket>> = HashMap::new();
    for pkt in iso {
        groups.entry(pkt.direction).or_default().push(pkt);
    }

    let mut segments = Vec::new();

    for (dir, pkts) in groups {
        if pkts.is_empty() { continue; }

        let mut current_packets: Vec<IsoPacket> = Vec::new();
        let mut current_iface: u8 = 0;
        let mut current_params: Option<descriptors::AudioParams> = None;

        for pkt in pkts {
            let params = ep_to_params.get(&pkt.endpoint).cloned();
            if let Some(ref cur_p) = current_params {
                if let Some(ref new_p) = params {
                    if !params_equal(cur_p, new_p) {
                        if !current_packets.is_empty() {
                            let seg = make_segment(&current_packets, current_iface, dir, current_params.take());
                            segments.push(seg);
                            current_packets = Vec::new();
                        }
                    }
                }
            }

            current_iface = pkt.interface;
            if current_params.is_none() {
                current_params = params;
            }
            current_packets.push((*pkt).clone());
        }

        if !current_packets.is_empty() {
            let seg = make_segment(&current_packets, current_iface, dir, current_params);
            segments.push(seg);
        }
    }

    segments
}

fn split_group(
    pkts: &[&IsoPacket],
    split_ts: &[f64],
    threshold_ms: u64,
    iface: u8,
    dir: Direction,
) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut current: Vec<IsoPacket> = Vec::new();
    let mut current_params: Option<descriptors::AudioParams> = None;

    for pkt in pkts {
        let mut needs_split = false;

        if let Some(last) = current.last() {
            let gap_ms = ((pkt.timestamp - last.timestamp) * 1000.0) as u64;
            if gap_ms > threshold_ms { needs_split = true; }
            else {
                let seg_end = last.timestamp;
                for &ts in split_ts {
                    if seg_end <= ts && ts <= pkt.timestamp && ts > seg_end {
                        needs_split = true;
                        break;
                    }
                }
            }
        }

        if needs_split && !current.is_empty() {
            let seg = make_segment(&current, iface, dir, current_params.take());
            segments.push(seg);
            current = Vec::new();
        }

        current.push((*pkt).clone());
        if current_params.is_none() {
            current_params = None;
        }
    }

    if !current.is_empty() {
        let seg = make_segment(&current, iface, dir, current_params);
        segments.push(seg);
    }

    segments
}

fn make_segment(packets: &[IsoPacket], iface: u8, dir: Direction, params: Option<descriptors::AudioParams>) -> Segment {
    let start = packets.first().map(|p| p.timestamp).unwrap_or(0.0);
    Segment {
        packets: packets.iter().map(|p| (*p).clone()).collect(),
        interface: iface,
        direction: dir,
        start_ts: start,
        audio_params: params,
    }
}

fn params_equal(a: &descriptors::AudioParams, b: &descriptors::AudioParams) -> bool {
    a.sample_rate == b.sample_rate && a.bit_depth == b.bit_depth && a.channels == b.channels
}

fn format_timestamp(ts: f64) -> String {
    let total_secs = ts as u64;
    let hours = (total_secs / 3600) % 24;
    let mins = (total_secs / 60) % 60;
    let secs = total_secs % 60;
    let millis = ((ts - total_secs as f64) * 1000.0) as u64;
    format!("{:02}{:02}{:02}_{:03}", hours, mins, secs, millis)
}
