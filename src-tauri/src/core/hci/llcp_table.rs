use super::types::{LlcpDef, ParamType};

pub fn lookup(opcode: u8) -> Option<&'static LlcpDef> {
    TABLE.iter().find(|l| l.opcode == opcode)
}

static TABLE: &[LlcpDef] = &[
    LlcpDef {
        opcode: 0x00,
        name: "LL_CONNECTION_UPDATE_IND",
        params: &[
            ("WinSize", ParamType::U8),
            ("WinOffset", ParamType::U16),
            ("Interval", ParamType::U16),
            ("Latency", ParamType::U16),
            ("Timeout", ParamType::U16),
            ("Instant", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x01,
        name: "LL_CHANNEL_MAP_IND",
        params: &[("ChM", ParamType::Bytes(5)), ("Instant", ParamType::U16)],
    },
    LlcpDef {
        opcode: 0x02,
        name: "LL_TERMINATE_IND",
        params: &[("ErrorCode", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x03,
        name: "LL_ENC_REQ",
        params: &[
            ("Rand", ParamType::Bytes(8)),
            ("EDIV", ParamType::U16),
            ("SKDm", ParamType::Bytes(8)),
            ("SKDs", ParamType::Bytes(8)),
        ],
    },
    LlcpDef {
        opcode: 0x04,
        name: "LL_ENC_RSP",
        params: &[("SKDs", ParamType::Bytes(8)), ("IVs", ParamType::Bytes(4))],
    },
    LlcpDef {
        opcode: 0x05,
        name: "LL_START_ENC_REQ",
        params: &[],
    },
    LlcpDef {
        opcode: 0x06,
        name: "LL_START_ENC_RSP",
        params: &[],
    },
    LlcpDef {
        opcode: 0x07,
        name: "LL_UNKNOWN_RSP",
        params: &[("UnknownType", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x08,
        name: "LL_FEATURE_REQ",
        params: &[("FeatureSet", ParamType::Bytes(8))],
    },
    LlcpDef {
        opcode: 0x09,
        name: "LL_FEATURE_RSP",
        params: &[("FeatureSet", ParamType::Bytes(8))],
    },
    LlcpDef {
        opcode: 0x0A,
        name: "LL_PAUSE_ENC_REQ",
        params: &[],
    },
    LlcpDef {
        opcode: 0x0B,
        name: "LL_PAUSE_ENC_RSP",
        params: &[],
    },
    LlcpDef {
        opcode: 0x0C,
        name: "LL_VERSION_IND",
        params: &[
            ("VersNr", ParamType::U8),
            ("CompId", ParamType::U16),
            ("SubVersNr", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x0D,
        name: "LL_REJECT_IND",
        params: &[("ErrorCode", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x0E,
        name: "LL_PERIPHERAL_FEATURE_REQ",
        params: &[("FeatureSet", ParamType::Bytes(8))],
    },
    LlcpDef {
        opcode: 0x0F,
        name: "LL_CONNECTION_PARAM_REQ",
        params: &[
            ("IntervalMin", ParamType::U16),
            ("IntervalMax", ParamType::U16),
            ("Latency", ParamType::U16),
            ("Timeout", ParamType::U16),
            ("PreferredPeriodicity", ParamType::U8),
            ("ReferenceConnEventCount", ParamType::U16),
            ("Offset0", ParamType::U16),
            ("Offset1", ParamType::U16),
            ("Offset2", ParamType::U16),
            ("Offset3", ParamType::U16),
            ("Offset4", ParamType::U16),
            ("Offset5", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x10,
        name: "LL_CONNECTION_PARAM_RSP",
        params: &[
            ("IntervalMin", ParamType::U16),
            ("IntervalMax", ParamType::U16),
            ("Latency", ParamType::U16),
            ("Timeout", ParamType::U16),
            ("PreferredPeriodicity", ParamType::U8),
            ("ReferenceConnEventCount", ParamType::U16),
            ("Offset0", ParamType::U16),
            ("Offset1", ParamType::U16),
            ("Offset2", ParamType::U16),
            ("Offset3", ParamType::U16),
            ("Offset4", ParamType::U16),
            ("Offset5", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x11,
        name: "LL_REJECT_EXT_IND",
        params: &[
            ("RejectOpcode", ParamType::U8),
            ("ErrorCode", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x12,
        name: "LL_PING_REQ",
        params: &[],
    },
    LlcpDef {
        opcode: 0x13,
        name: "LL_PING_RSP",
        params: &[],
    },
    LlcpDef {
        opcode: 0x14,
        name: "LL_LENGTH_REQ",
        params: &[
            ("MaxRxOctets", ParamType::U16),
            ("MaxRxTime", ParamType::U16),
            ("MaxTxOctets", ParamType::U16),
            ("MaxTxTime", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x15,
        name: "LL_LENGTH_RSP",
        params: &[
            ("MaxRxOctets", ParamType::U16),
            ("MaxRxTime", ParamType::U16),
            ("MaxTxOctets", ParamType::U16),
            ("MaxTxTime", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x16,
        name: "LL_PHY_REQ",
        params: &[("TX_PHYS", ParamType::U8), ("RX_PHYS", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x17,
        name: "LL_PHY_RSP",
        params: &[("TX_PHYS", ParamType::U8), ("RX_PHYS", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x18,
        name: "LL_PHY_UPDATE_IND",
        params: &[
            ("PHY_C_To_P", ParamType::U8),
            ("PHY_P_To_C", ParamType::U8),
            ("Instant", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x19,
        name: "LL_MIN_USED_CHANNELS_IND",
        params: &[("PHYs", ParamType::U8), ("MinUsedChannels", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x1A,
        name: "LL_CTE_REQ",
        params: &[
            ("MinCTELenReq", ParamType::U8),
            ("CTETypeReq", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x1B,
        name: "LL_CTE_RSP",
        params: &[],
    },
    LlcpDef {
        opcode: 0x1C,
        name: "LL_PERIODIC_SYNC_IND",
        params: &[
            ("ID", ParamType::U16),
            ("SyncInfo", ParamType::Bytes(18)),
            ("ID_Type", ParamType::U8),
            ("Advertising_SID", ParamType::U8),
            ("Advertising_Address", ParamType::BdAddr),
            ("SyncConnEventCount", ParamType::U16),
            ("LastPaEventCount", ParamType::U16),
            ("SID", ParamType::U8),
            ("A_SCA", ParamType::U8),
            ("PHY", ParamType::U8),
            ("AdvA", ParamType::BdAddr),
            ("SyncSkip", ParamType::U16),
            ("SyncTimeout", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x1D,
        name: "LL_CLOCK_ACCURACY_REQ",
        params: &[("SCA", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x1E,
        name: "LL_CLOCK_ACCURACY_RSP",
        params: &[("SCA", ParamType::U8)],
    },
    LlcpDef {
        opcode: 0x1F,
        name: "LL_CIS_REQ",
        params: &[
            ("CIG_ID", ParamType::U8),
            ("CIS_ID", ParamType::U8),
            ("PHY_C_To_P", ParamType::U8),
            ("PHY_P_To_C", ParamType::U8),
            ("MaxSDU_C_To_P", ParamType::U16),
            ("MaxSDU_P_To_C", ParamType::U16),
            ("SDU_Interval_C_To_P", ParamType::U24),
            ("SDU_Interval_P_To_C", ParamType::U24),
            ("FT_C_To_P", ParamType::U8),
            ("FT_P_To_C", ParamType::U8),
            ("ISO_Interval", ParamType::U16),
            ("NSE", ParamType::U8),
            ("SubInterval_C_To_P", ParamType::U24),
            ("SubInterval_P_To_C", ParamType::U24),
            ("BN_C_To_P", ParamType::U8),
            ("BN_P_To_C", ParamType::U8),
            ("Max_PDU_C_To_P", ParamType::U8),
            ("Max_PDU_P_To_C", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x20,
        name: "LL_CIS_RSP",
        params: &[
            ("CIS_Offset_Min", ParamType::U24),
            ("CIS_Offset_Max", ParamType::U24),
            ("CIS_Interval_Min", ParamType::U16),
            ("CIS_Interval_Max", ParamType::U16),
            ("SubInterval_C_To_P", ParamType::U24),
            ("SubInterval_P_To_C", ParamType::U24),
            ("NCE", ParamType::U8),
            ("BN_C_To_P", ParamType::U8),
            ("BN_P_To_C", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x21,
        name: "LL_CIS_IND",
        params: &[
            ("AccessAddress", ParamType::U32),
            ("CIS_Offset", ParamType::U24),
            ("CIG_Sync_Delay", ParamType::U24),
            ("CIS_Sync_Delay", ParamType::U24),
        ],
    },
    LlcpDef {
        opcode: 0x22,
        name: "LL_CIS_TERMINATE_IND",
        params: &[
            ("CIG_ID", ParamType::U8),
            ("CIS_ID", ParamType::U8),
            ("ErrorCode", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x23,
        name: "LL_POWER_CONTROL_REQ",
        params: &[
            ("PHY", ParamType::U8),
            ("Delta", ParamType::U8),
            ("TxPower", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x24,
        name: "LL_POWER_CONTROL_RSP",
        params: &[
            ("Flags", ParamType::U8),
            ("Delta", ParamType::U8),
            ("TxPower", ParamType::U8),
            ("APA", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x25,
        name: "LL_POWER_CHANGE_IND",
        params: &[
            ("PHY", ParamType::U8),
            ("Flags", ParamType::U8),
            ("Delta", ParamType::U8),
            ("TxPower", ParamType::U8),
        ],
    },
    LlcpDef {
        opcode: 0x26,
        name: "LL_SUBRATE_REQ",
        params: &[
            ("SubrateFactorMin", ParamType::U16),
            ("SubrateFactorMax", ParamType::U16),
            ("MaxLatency", ParamType::U16),
            ("ContinuationNumber", ParamType::U16),
            ("SupervisionTimeout", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x27,
        name: "LL_SUBRATE_IND",
        params: &[
            ("SubrateFactor", ParamType::U16),
            ("PeripheralLatency", ParamType::U16),
            ("ContinuationNumber", ParamType::U16),
            ("SupervisionTimeout", ParamType::U16),
            ("Instant", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x28,
        name: "LL_CHANNEL_REPORTING_IND",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x29,
        name: "LL_CHANNEL_STATUS_IND",
        params: &[("Channel_Status", ParamType::Bytes(5))],
    },
    LlcpDef {
        opcode: 0x2A,
        name: "LL_PERIODIC_SYNC_WR_IND",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x2B,
        name: "LL_FEATURE_EXT_REQ",
        params: &[
            ("FeaturePage", ParamType::U8),
            ("MaxSupportedPage", ParamType::U8),
            ("FeatureSet", ParamType::Bytes(8)),
        ],
    },
    LlcpDef {
        opcode: 0x2C,
        name: "LL_FEATURE_EXT_RSP",
        params: &[
            ("FeaturePage", ParamType::U8),
            ("MaxSupportedPage", ParamType::U8),
            ("FeatureSet", ParamType::Bytes(8)),
        ],
    },
    LlcpDef {
        opcode: 0x2D,
        name: "LL_CS_SEC_RSP",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x2E,
        name: "LL_CS_CAPABILITIES_REQ",
        params: &[],
    },
    LlcpDef {
        opcode: 0x2F,
        name: "LL_CS_CAPABILITIES_RSP",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x30,
        name: "LL_CS_CONFIG_REQ",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x31,
        name: "LL_CS_CONFIG_RSP",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x32,
        name: "LL_CS_REQ",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x33,
        name: "LL_CS_RSP",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x34,
        name: "LL_CS_IND",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x35,
        name: "LL_CS_TERMINATE_REQ",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x36,
        name: "LL_CS_FAE_REQ",
        params: &[],
    },
    LlcpDef {
        opcode: 0x37,
        name: "LL_CS_FAE_RSP",
        params: &[("Remote_FAE_Table", ParamType::Bytes(72))],
    },
    LlcpDef {
        opcode: 0x38,
        name: "LL_CS_CHANNEL_MAP_IND",
        params: &[
            ("Channel_Map", ParamType::Bytes(10)),
            ("Instant", ParamType::U16),
        ],
    },
    LlcpDef {
        opcode: 0x39,
        name: "LL_CS_SEC_REQ",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x3A,
        name: "LL_CS_TERMINATE_RSP",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    LlcpDef {
        opcode: 0x3B,
        name: "LL_FRAME_SPACE_REQ",
        params: &[("Frame_Space", ParamType::U16)],
    },
    LlcpDef {
        opcode: 0x3C,
        name: "LL_FRAME_SPACE_RSP",
        params: &[("Frame_Space", ParamType::U16)],
    },
];
