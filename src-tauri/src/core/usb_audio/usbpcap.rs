pub const URB_ISOCHRONOUS: u16 = 0x000A;
pub const URB_CONTROL: u16 = 0x0001;
pub const URB_GET_DESCRIPTOR: u16 = 0x0008;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction { In, Out }

#[derive(Debug, Clone)]
pub struct IsoPacket {
    pub timestamp: f64,
    pub endpoint: u8,
    pub direction: Direction,
    pub interface: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SetInterfaceEvent {
    pub timestamp: f64,
    pub interface: u8,
    pub alt_setting: u8,
}

pub enum Classified {
    Iso(IsoPacket),
    SetInterface(SetInterfaceEvent),
    DescriptorResponse(Vec<u8>),
    Other,
}

pub fn classify(raw: &[u8], timestamp: f64) -> Classified {
    if raw.len() < 25 { return Classified::Other; }

    let hdr_len = u16::from_le_bytes([raw[0], raw[1]]) as usize;
    let func = u16::from_le_bytes([raw[14], raw[15]]);
    let irp_info = raw[16];
    let endpoint = raw[21];
    let transfer_type = raw[22];

    match func {
        URB_ISOCHRONOUS if transfer_type == 0 => {
            let data_len = u16::from_le_bytes([raw[23], raw[24]]) as usize;
            let direction = if endpoint & 0x80 != 0 { Direction::In } else { Direction::Out };
            let valid = match direction {
                Direction::In => irp_info == 0x01,
                Direction::Out => irp_info == 0x00,
            };
            if valid && hdr_len + data_len <= raw.len() {
                let data = raw[hdr_len..hdr_len + data_len].to_vec();
                return Classified::Iso(IsoPacket { timestamp, endpoint, direction, interface: 0, data });
            }
        }
        URB_CONTROL if irp_info == 0x00 => {
            if hdr_len + 8 <= raw.len() {
                let setup = &raw[hdr_len..hdr_len + 8];
                if setup[1] == 0x0B {
                    let alt_setting = setup[2];
                    let interface = setup[4];
                    let _wvalue = u16::from_le_bytes([setup[2], setup[3]]);
                    let _windex = u16::from_le_bytes([setup[4], setup[5]]);
                    return Classified::SetInterface(SetInterfaceEvent {
                        timestamp,
                        interface,
                        alt_setting,
                    });
                }
            }
        }
        URB_GET_DESCRIPTOR if irp_info == 0x01 => {
            let data_len = u16::from_le_bytes([raw[23], raw[24]]) as usize;
            if hdr_len + data_len <= raw.len() {
                return Classified::DescriptorResponse(raw[hdr_len..hdr_len + data_len].to_vec());
            }
        }
        URB_CONTROL if irp_info == 0x01 => {
            // Generic control transfer IN data stage (e.g. GET_DESCRIPTOR via URB_FUNCTION_CONTROL_TRANSFER)
            let payload_len = raw.len() - hdr_len;
            if payload_len > 0 {
                return Classified::DescriptorResponse(raw[hdr_len..].to_vec());
            }
        }
        _ => {}
    }
    Classified::Other
}
