use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AudioParams {
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub channels: u8,
}

#[derive(Debug, Clone)]
pub struct EndpointInfo {
    pub interface: u8,
    pub alt_setting: u8,
    pub params: Option<AudioParams>,
}

pub fn parse_audio_descriptors(descriptor_blobs: &[Vec<u8>]) -> HashMap<u8, EndpointInfo> {
    let mut result: HashMap<u8, EndpointInfo> = HashMap::new();

    for blob in descriptor_blobs {
        let parsed = parse_descriptor_chain(blob);
        for (ep, mut info) in parsed {
            if let Some(existing) = result.get(&ep) {
                if existing.params.is_some() && info.params.is_none() {
                    continue;
                }
            }
            if let Some(ref mut existing) = result.get_mut(&ep) {
                if info.params.is_some() {
                    existing.params = info.params.take();
                }
            } else {
                result.insert(ep, info);
            }
        }
    }

    result
}

fn parse_descriptor_chain(data: &[u8]) -> HashMap<u8, EndpointInfo> {
    let mut result: HashMap<u8, EndpointInfo> = HashMap::new();
    let mut current_iface: u8 = 0;
    let mut current_alt: u8 = 0;
    let mut current_audio_params: Option<AudioParams> = None;
    let mut offset = 0;

    while offset + 2 <= data.len() {
        let desc_len = data[offset] as usize;
        let desc_type = data[offset + 1];

        if desc_len < 2 || offset + desc_len > data.len() { break; }

        match desc_type {
            4 => {
                if desc_len >= 9 {
                    current_iface = data[offset + 2];
                    current_alt = data[offset + 3];
                }
                current_audio_params = None;
            }
            5 => {
                if desc_len >= 7 {
                    let ep_addr = data[offset + 2];
                    result.entry(ep_addr).or_insert(EndpointInfo {
                        interface: current_iface,
                        alt_setting: current_alt,
                        params: current_audio_params.clone(),
                    });
                }
            }
            0x24 => {
                let subtype = data[offset + 2];
                if subtype == 2 && desc_len >= 11 {
                    let channels = data[offset + 4];
                    let sub_slot_size = data[offset + 5];
                    let bit_depth = data[offset + 6];
                    let effective_bits = if bit_depth == 0 { sub_slot_size * 8 } else { bit_depth };
                    if offset + 11 <= desc_len + offset {
                        let sr = u32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], 0]);
                        if sr > 0 {
                            current_audio_params = Some(AudioParams {
                                sample_rate: sr,
                                bit_depth: effective_bits,
                                channels,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        offset += desc_len;
    }

    result
}
