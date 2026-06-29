use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParsedProtocol {
    pub protocol: String,
    pub name: String,
    pub opcode_info: String,
    pub recognized: bool,
    pub fields: Vec<ProtocolField>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtocolField {
    pub name: String,
    pub value: String,
}

pub enum ParamType {
    U8,
    U16,
    U24,
    U32,
    BdAddr,
    Bytes(usize),
}

pub struct HciCmdDef {
    pub ogf: u8,
    pub ocf: u16,
    pub name: &'static str,
    pub params: &'static [(&'static str, ParamType)],
}

pub struct HciEvtDef {
    pub code: u8,
    pub name: &'static str,
    pub params: &'static [(&'static str, ParamType)],
}

pub struct LmpDef {
    pub opcode: u8,
    pub name: &'static str,
    pub params: &'static [(&'static str, ParamType)],
}

pub struct LlcpDef {
    pub opcode: u8,
    pub name: &'static str,
    pub params: &'static [(&'static str, ParamType)],
}
