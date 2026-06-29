use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    SUB,
    CHK,
    LVL,
    LST,
    LSV,
    U08,
    S08,
    U16,
    UBT,
    TXT,
    MAC,
}

impl EntryType {
    pub fn from_str(s: &str) -> Option<EntryType> {
        match s {
            "SUB" => Some(EntryType::SUB),
            "CHK" => Some(EntryType::CHK),
            "LVL" => Some(EntryType::LVL),
            "LST" => Some(EntryType::LST),
            "LSV" => Some(EntryType::LSV),
            "U08" => Some(EntryType::U08),
            "S08" => Some(EntryType::S08),
            "U16" => Some(EntryType::U16),
            "UBT" => Some(EntryType::UBT),
            "TXT" => Some(EntryType::TXT),
            "MAC" => Some(EntryType::MAC),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    pub label: String,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    String(String),
    Mac(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub entry_type: EntryType,
    pub label_cn: String,
    pub name: String,
    pub tooltip: String,
    pub var_name: String,

    // 内存布局
    pub offset: i32,
    pub bit_offset: u8,
    pub bit_width: u8,
    pub size: u8,

    // 值相关
    pub value: Option<ConfigValue>,
    pub default_value: Option<ConfigValue>,
    pub min_val: i32,
    pub max_val: i32,

    // 列表类型
    pub options: Vec<ConfigOption>,
    pub val_type: u16,

    // MAC 类型
    pub addr_start: Option<[u8; 6]>,
    pub addr_end: Option<[u8; 6]>,
    pub addr_set: Option<[u8; 6]>,

    // TXT 类型
    pub str_length: u8,

    // 树形结构
    pub children: Vec<ConfigItem>,
    pub level_value: u32,
    pub ui_condition_var: Option<String>,
}

impl Default for ConfigItem {
    fn default() -> Self {
        Self {
            entry_type: EntryType::SUB,
            label_cn: String::new(),
            name: String::new(),
            tooltip: String::new(),
            var_name: String::new(),
            offset: -1,
            bit_offset: 0,
            bit_width: 0,
            size: 0,
            value: None,
            default_value: None,
            min_val: 0,
            max_val: 0,
            options: Vec::new(),
            val_type: 0,
            addr_start: None,
            addr_end: None,
            addr_set: None,
            str_length: 0,
            children: Vec::new(),
            level_value: 0,
            ui_condition_var: None,
        }
    }
}
