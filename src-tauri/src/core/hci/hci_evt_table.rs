use super::types::{HciEvtDef, ParamType};

pub fn lookup(code: u8) -> Option<&'static HciEvtDef> {
    TABLE.iter().find(|e| e.code == code)
}

static TABLE: &[HciEvtDef] = &[
    HciEvtDef {
        code: 0x01,
        name: "Inquiry_Complete",
        params: &[("Status", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x02,
        name: "Inquiry_Result",
        params: &[("Num_Responses", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x03,
        name: "Connection_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("BD_ADDR", ParamType::BdAddr),
            ("Link_Type", ParamType::U8),
            ("Encryption_Enabled", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x04,
        name: "Connection_Request",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Class_of_Device", ParamType::U24),
            ("Link_Type", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x05,
        name: "Disconnection_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Reason", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x06,
        name: "Authentication_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x07,
        name: "Remote_Name_Request_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("BD_ADDR", ParamType::BdAddr),
            ("Remote_Name", ParamType::Bytes(248)),
        ],
    },
    HciEvtDef {
        code: 0x08,
        name: "Encryption_Change",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Encryption_Enabled", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x09,
        name: "Change_Connection_Link_Key_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x0A,
        name: "Link_Key_Type_Changed",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Key_Type", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x0B,
        name: "Read_Remote_Supported_Features_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("LMP_Features", ParamType::Bytes(8)),
        ],
    },
    HciEvtDef {
        code: 0x0C,
        name: "Read_Remote_Version_Information_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Version", ParamType::U8),
            ("Manufacturer_Name", ParamType::U16),
            ("Subversion", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x0D,
        name: "QoS_Setup_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Flags", ParamType::U8),
            ("Service_Type", ParamType::U8),
            ("Token_Rate", ParamType::U32),
            ("Peak_Bandwidth", ParamType::U32),
            ("Latency", ParamType::U32),
            ("Delay_Variation", ParamType::U32),
        ],
    },
    HciEvtDef {
        code: 0x0E,
        name: "Command_Complete",
        params: &[
            ("Num_HCI_Command_Packets", ParamType::U8),
            ("Command_Opcode", ParamType::U16),
            ("Return_Parameters", ParamType::Bytes(0)),
        ],
    },
    HciEvtDef {
        code: 0x0F,
        name: "Command_Status",
        params: &[
            ("Status", ParamType::U8),
            ("Num_HCI_Command_Packets", ParamType::U8),
            ("Command_Opcode", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x10,
        name: "Hardware_Error",
        params: &[("Hardware_Code", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x11,
        name: "Flush_Occurred",
        params: &[("Connection_Handle", ParamType::U16)],
    },
    HciEvtDef {
        code: 0x12,
        name: "Role_Change",
        params: &[
            ("Status", ParamType::U8),
            ("BD_ADDR", ParamType::BdAddr),
            ("New_Role", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x13,
        name: "Number_Of_Completed_Packets",
        params: &[("Num_Handles", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x14,
        name: "Mode_Change",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Current_Mode", ParamType::U8),
            ("Interval", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x15,
        name: "Return_Link_Keys",
        params: &[("Num_Keys", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x16,
        name: "PIN_Code_Request",
        params: &[("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x17,
        name: "Link_Key_Request",
        params: &[("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x18,
        name: "Link_Key_Notification",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Link_Key", ParamType::Bytes(16)),
            ("Key_Type", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x19,
        name: "Loopback_Command",
        params: &[],
    },
    HciEvtDef {
        code: 0x1A,
        name: "Data_Buffer_Overflow",
        params: &[("Link_Type", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x1B,
        name: "Max_Slots_Change",
        params: &[
            ("Connection_Handle", ParamType::U16),
            ("LMP_Max_Slots", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x1C,
        name: "Read_Clock_Offset_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Clock_Offset", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x1D,
        name: "Connection_Packet_Type_Changed",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Packet_Type", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x1E,
        name: "QoS_Violation",
        params: &[("Connection_Handle", ParamType::U16)],
    },
    HciEvtDef {
        code: 0x20,
        name: "Page_Scan_Repetition_Mode_Change",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Page_Scan_Repetition_Mode", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x21,
        name: "Flow_Specification_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Flags", ParamType::U8),
            ("Flow_Direction", ParamType::U8),
            ("Service_Type", ParamType::U8),
            ("Token_Rate", ParamType::U32),
            ("Token_Bucket_Size", ParamType::U32),
            ("Peak_Bandwidth", ParamType::U32),
            ("Access_Latency", ParamType::U32),
        ],
    },
    HciEvtDef {
        code: 0x22,
        name: "Inquiry_Result_With_RSSI",
        params: &[("Num_Responses", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x23,
        name: "Read_Remote_Extended_Features_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Page_Number", ParamType::U8),
            ("Max_Page_Number", ParamType::U8),
            ("Extended_LMP_Features", ParamType::Bytes(8)),
        ],
    },
    HciEvtDef {
        code: 0x2C,
        name: "Synchronous_Connection_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("BD_ADDR", ParamType::BdAddr),
            ("Link_Type", ParamType::U8),
            ("Transmission_Interval", ParamType::U8),
            ("Retransmission_Window", ParamType::U8),
            ("RX_Packet_Length", ParamType::U16),
            ("TX_Packet_Length", ParamType::U16),
            ("Air_Mode", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x2D,
        name: "Synchronous_Connection_Changed",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Transmission_Interval", ParamType::U8),
            ("Retransmission_Window", ParamType::U8),
            ("RX_Packet_Length", ParamType::U16),
            ("TX_Packet_Length", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x2E,
        name: "Sniff_Subrating",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Max_Tx_Latency", ParamType::U16),
            ("Max_Rx_Latency", ParamType::U16),
            ("Min_Remote_Timeout", ParamType::U16),
            ("Min_Local_Timeout", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x2F,
        name: "Extended_Inquiry_Result",
        params: &[("Num_Responses", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x30,
        name: "Encryption_Key_Refresh_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x31,
        name: "IO_Capability_Request",
        params: &[("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x32,
        name: "IO_Capability_Response",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("IO_Capability", ParamType::U8),
            ("OOB_Data_Present", ParamType::U8),
            ("Authentication_Requirements", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x33,
        name: "User_Confirmation_Request",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Numeric_Value", ParamType::U32),
        ],
    },
    HciEvtDef {
        code: 0x34,
        name: "User_Passkey_Request",
        params: &[("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x35,
        name: "Remote_OOB_Data_Request",
        params: &[("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x36,
        name: "Simple_Pairing_Complete",
        params: &[("Status", ParamType::U8), ("BD_ADDR", ParamType::BdAddr)],
    },
    HciEvtDef {
        code: 0x38,
        name: "Link_Supervision_Timeout_Changed",
        params: &[
            ("Connection_Handle", ParamType::U16),
            ("Link_Supervision_Timeout", ParamType::U16),
        ],
    },
    HciEvtDef {
        code: 0x39,
        name: "Enhanced_Flush_Complete",
        params: &[("Connection_Handle", ParamType::U16)],
    },
    HciEvtDef {
        code: 0x3B,
        name: "User_Passkey_Notification",
        params: &[("BD_ADDR", ParamType::BdAddr), ("Passkey", ParamType::U32)],
    },
    HciEvtDef {
        code: 0x3C,
        name: "Keypress_Notification",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Notification_Type", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x3D,
        name: "Remote_Host_Supported_Features_Notification",
        params: &[
            ("BD_ADDR", ParamType::BdAddr),
            ("Host_Supported_Features", ParamType::Bytes(8)),
        ],
    },
    HciEvtDef {
        code: 0x3E,
        name: "LE_Meta_Event",
        params: &[("Subevent_Code", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x40,
        name: "Physical_Link_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Physical_Link_Handle", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x41,
        name: "Channel_Selected",
        params: &[("Physical_Link_Handle", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x42,
        name: "Disconnection_Physical_Link_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Physical_Link_Handle", ParamType::U8),
            ("Reason", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x45,
        name: "Logical_Link_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Logical_Link_Handle", ParamType::U16),
            ("Physical_Link_Handle", ParamType::U8),
            ("TX_Flow_Spec_ID", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x46,
        name: "Disconnection_Logical_Link_Complete",
        params: &[
            ("Status", ParamType::U8),
            ("Logical_Link_Handle", ParamType::U16),
            ("Reason", ParamType::U8),
        ],
    },
    HciEvtDef {
        code: 0x48,
        name: "Number_Of_Completed_Data_Blocks",
        params: &[("Num_Handles", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x4F,
        name: "Synchronization_Train_Complete",
        params: &[("Status", ParamType::U8)],
    },
    HciEvtDef {
        code: 0x54,
        name: "Peripheral_Page_Response_Timeout",
        params: &[],
    },
    HciEvtDef {
        code: 0x57,
        name: "Authenticated_Payload_Timeout_Expired",
        params: &[("Connection_Handle", ParamType::U16)],
    },
    HciEvtDef {
        code: 0x58,
        name: "SAM_Status_Change",
        params: &[("Parameters", ParamType::Bytes(0))],
    },
    HciEvtDef {
        code: 0x59,
        name: "Encryption_Change_V2",
        params: &[
            ("Status", ParamType::U8),
            ("Connection_Handle", ParamType::U16),
            ("Encryption_Enabled", ParamType::U8),
            ("Key_Type", ParamType::U8),
        ],
    },
];
