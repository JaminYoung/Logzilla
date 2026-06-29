use super::types::{HciCmdDef, ParamType};

pub fn lookup(ogf: u8, ocf: u16) -> Option<&'static HciCmdDef> {
    TABLE.iter().find(|c| c.ogf == ogf && c.ocf == ocf)
}

static TABLE: &[HciCmdDef] = &[
    // OGF=0x01 Link Control
    HciCmdDef { ogf: 0x01, ocf: 0x0001, name: "Inquiry", params: &[
        ("LAP", ParamType::U24),
        ("Inquiry_Length", ParamType::U8),
        ("Num_Responses", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0002, name: "Inquiry_Cancel", params: &[] },
    HciCmdDef { ogf: 0x01, ocf: 0x0003, name: "Periodic_Inquiry_Mode", params: &[
        ("Max_Period_Length", ParamType::U16),
        ("Min_Period_Length", ParamType::U16),
        ("LAP", ParamType::U24),
        ("Inquiry_Length", ParamType::U8),
        ("Num_Responses", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0004, name: "Exit_Periodic_Inquiry_Mode", params: &[] },
    HciCmdDef { ogf: 0x01, ocf: 0x0005, name: "Create_Connection", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Packet_Type", ParamType::U16),
        ("Page_Scan_Repetition_Mode", ParamType::U8),
        ("Reserved", ParamType::U8),
        ("Clock_Offset", ParamType::U16),
        ("Allow_Role_Switch", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0006, name: "Disconnect", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Reason", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0008, name: "Create_Connection_Cancel", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0009, name: "Accept_Connection_Request", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Role", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000A, name: "Reject_Connection_Request", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Reason", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000B, name: "Link_Key_Request_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Link_Key", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000C, name: "Link_Key_Request_Negative_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000D, name: "PIN_Code_Request_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("PIN_Code_Length", ParamType::U8),
        ("PIN_Code", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000E, name: "PIN_Code_Request_Negative_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x000F, name: "Change_Connection_Packet_Type", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Packet_Type", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0011, name: "Authentication_Requested", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0013, name: "Set_Connection_Encryption", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Encryption_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0015, name: "Change_Connection_Link_Key", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0017, name: "Link_Key_Selection", params: &[
        ("Key_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0019, name: "Remote_Name_Request", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Page_Scan_Repetition_Mode", ParamType::U8),
        ("Reserved", ParamType::U8),
        ("Clock_Offset", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x001A, name: "Remote_Name_Request_Cancel", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x001B, name: "Read_Remote_Supported_Features", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x001C, name: "Read_Remote_Extended_Features", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Page_Number", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x001D, name: "Read_Remote_Version_Information", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0020, name: "Read_LMP_Handle", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0028, name: "Setup_Synchronous_Connection", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Transmit_Bandwidth", ParamType::U32),
        ("Receive_Bandwidth", ParamType::U32),
        ("Max_Latency", ParamType::U16),
        ("Voice_Setting", ParamType::U16),
        ("Retransmission_Effort", ParamType::U8),
        ("Packet_Type", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0029, name: "Accept_Synchronous_Connection_Request", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Transmit_Bandwidth", ParamType::U32),
        ("Receive_Bandwidth", ParamType::U32),
        ("Max_Latency", ParamType::U16),
        ("Content_Format", ParamType::U16),
        ("Retransmission_Effort", ParamType::U8),
        ("Packet_Type", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x002A, name: "Reject_Synchronous_Connection_Request", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Reason", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x002B, name: "IO_Capability_Request_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("IO_Capability", ParamType::U8),
        ("OOB_Data_Present", ParamType::U8),
        ("Authentication_Requirements", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x002C, name: "User_Confirmation_Request_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x002D, name: "User_Confirmation_Request_Negative_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x01, ocf: 0x0034, name: "IO_Capability_Request_Negative_Reply", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Reason", ParamType::U8),
    ]},

    // OGF=0x02 Link Policy (per Core Spec)
    HciCmdDef { ogf: 0x02, ocf: 0x0001, name: "Hold_Mode", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Hold_Mode_Max_Interval", ParamType::U16),
        ("Hold_Mode_Min_Interval", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0003, name: "Sniff_Mode", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Sniff_Max_Interval", ParamType::U16),
        ("Sniff_Min_Interval", ParamType::U16),
        ("Sniff_Attempt", ParamType::U16),
        ("Sniff_Timeout", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0004, name: "Exit_Sniff_Mode", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0007, name: "QoS_Setup", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Flags", ParamType::U8),
        ("Service_Type", ParamType::U8),
        ("Token_Rate", ParamType::U32),
        ("Peak_Bandwidth", ParamType::U32),
        ("Latency", ParamType::U32),
        ("Delay_Variation", ParamType::U32),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0009, name: "Role_Discovery", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x000B, name: "Switch_Role", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Role", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x000C, name: "Read_Link_Policy_Settings", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x000D, name: "Write_Link_Policy_Settings", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Link_Policy_Settings", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x000E, name: "Read_Default_Link_Policy_Settings", params: &[] },
    HciCmdDef { ogf: 0x02, ocf: 0x000F, name: "Write_Default_Link_Policy_Settings", params: &[
        ("Link_Policy_Settings", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0010, name: "Flow_Specification", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Flags", ParamType::U8),
        ("Flow_Direction", ParamType::U8),
        ("Service_Type", ParamType::U8),
        ("Token_Rate", ParamType::U32),
        ("Token_Bucket_Size", ParamType::U32),
        ("Peak_Bandwidth", ParamType::U32),
        ("Access_Latency", ParamType::U32),
    ]},
    HciCmdDef { ogf: 0x02, ocf: 0x0011, name: "Sniff_Subrating", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Max_Latency", ParamType::U16),
        ("Min_Remote_Timeout", ParamType::U16),
        ("Min_Local_Timeout", ParamType::U16),
    ]},

    // OGF=0x03 Host Controller & Baseband
    HciCmdDef { ogf: 0x03, ocf: 0x0001, name: "Set_Event_Mask", params: &[
        ("Event_Mask", ParamType::Bytes(8)),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0003, name: "Reset", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0005, name: "Set_Event_Filter", params: &[
        ("Filter_Type", ParamType::U8),
        ("Filter_Condition_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0008, name: "Flush", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0009, name: "Read_PIN_Type", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x000A, name: "Write_PIN_Type", params: &[
        ("PIN_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x000B, name: "Read_Local_Name", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x000C, name: "Write_Local_Name", params: &[
        ("Local_Name", ParamType::Bytes(248)),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x000D, name: "Read_Stored_Link_Key", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Read_All", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x000E, name: "Delete_Stored_Link_Key", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Delete_All", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x000F, name: "Write_Connection_Accept_Timeout", params: &[
        ("Conn_Accept_Timeout", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0010, name: "Read_Connection_Accept_Timeout", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0011, name: "Read_Page_Timeout", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0012, name: "Write_Page_Scan_Activity", params: &[
        ("Page_Scan_Interval", ParamType::U16),
        ("Page_Scan_Window", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0013, name: "Read_Inquiry_Scan_Activity", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0014, name: "Write_Inquiry_Scan_Activity", params: &[
        ("Inquiry_Scan_Interval", ParamType::U16),
        ("Inquiry_Scan_Window", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0015, name: "Read_Authentication_Enable", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0016, name: "Write_Authentication_Enable", params: &[
        ("Authentication_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0017, name: "Read_Page_Timeout", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0018, name: "Write_Page_Timeout", params: &[
        ("Page_Timeout", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0019, name: "Read_Scan_Enable", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x001A, name: "Write_Scan_Enable", params: &[
        ("Scan_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x001B, name: "Read_Class_of_Device", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x001C, name: "Write_Class_of_Device", params: &[
        ("Class_of_Device", ParamType::U24),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x001D, name: "Read_Voice_Setting", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x001E, name: "Write_Voice_Setting", params: &[
        ("Voice_Setting", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0023, name: "Read_Page_Scan_Type", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0024, name: "Write_Page_Scan_Type", params: &[
        ("Page_Scan_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0025, name: "Read_Inquiry_Scan_Type", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0026, name: "Write_Inquiry_Scan_Type", params: &[
        ("Inquiry_Scan_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0027, name: "Read_Inquiry_Mode", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0028, name: "Write_Inquiry_Mode", params: &[
        ("Inquiry_Mode", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0029, name: "Read_Page_Scan_Mode", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x002A, name: "Write_Page_Scan_Mode", params: &[
        ("Page_Scan_Mode", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x002B, name: "Read_Auto_Flush_Timeout", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x002C, name: "Write_Auto_Flush_Timeout", params: &[
        ("Handle", ParamType::U16),
        ("Flush_Timeout", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x002D, name: "Read_Num_Broadcast_Retransmissions", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x002E, name: "Write_Num_Broadcast_Retransmissions", params: &[
        ("Num_Broadcast_Retransmissions", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x002F, name: "Read_Hold_Mode_Activity", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0030, name: "Write_Hold_Mode_Activity", params: &[
        ("Hold_Mode_Activity", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0031, name: "Read_Transmit_Power_Level", params: &[
        ("Handle", ParamType::U16),
        ("Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0032, name: "Read_Synchronous_Flow_Control_Enable", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0033, name: "Host_Buffer_Size", params: &[
        ("Host_ACL_Data_Packet_Length", ParamType::U16),
        ("Host_Synchronous_Data_Packet_Length", ParamType::U8),
        ("Host_Total_Num_ACL_Data_Packets", ParamType::U16),
        ("Host_Total_Num_Synchronous_Data_Packets", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0035, name: "Read_Link_Supervision_Timeout", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0036, name: "Write_Link_Supervision_Timeout", params: &[
        ("Handle", ParamType::U16),
        ("Link_Supervision_Timeout", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0037, name: "Read_Number_Of_Supported_IAC", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0038, name: "Read_Current_IAC_LAP", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0039, name: "Write_Current_IAC_LAP", params: &[
        ("Num_Current_IAC", ParamType::U8),
        ("IAC_LAP", ParamType::Bytes(0)),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0045, name: "Read_Inquiry_Response_Power_Level", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0046, name: "Write_Inquiry_Transmit_Power_Level", params: &[
        ("TX_Power_Level", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0047, name: "Read_Default_Erroneous_Data_Reporting", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0051, name: "Enhanced_Flush", params: &[
        ("Handle", ParamType::U16),
        ("Packet_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0052, name: "Write_Extended_Inquiry_Response", params: &[
        ("FEC_Required", ParamType::U8),
        ("EIR_Data", ParamType::Bytes(240)),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0060, name: "Send_Keypress_Notification", params: &[
        ("BD_ADDR", ParamType::BdAddr),
        ("Notification_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0055, name: "Set_Event_Mask_Page_2", params: &[
        ("Event_Mask_Page_2", ParamType::Bytes(8)),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0056, name: "Write_Simple_Pairing_Mode", params: &[
        ("Simple_Pairing_Mode", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x0057, name: "Read_Local_OOB_Data", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0058, name: "Read_Flow_Control_Mode", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0063, name: "Read_Secure_Connections_Host_Support", params: &[] },
    HciCmdDef { ogf: 0x03, ocf: 0x0064, name: "Write_Secure_Connections_Host_Support", params: &[
        ("Secure_Connections_Host_Support", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x006C, name: "Read_Authenticated_Payload_Timeout", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x03, ocf: 0x006D, name: "Write_Authenticated_Payload_Timeout", params: &[
        ("Handle", ParamType::U16),
        ("Authenticated_Payload_Timeout", ParamType::U16),
    ]},

    // OGF=0x04 Informational Parameters
    HciCmdDef { ogf: 0x04, ocf: 0x0001, name: "Read_Local_Version_Information", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x0002, name: "Read_Local_Supported_Commands", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x0003, name: "Read_Local_Supported_Features", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x0004, name: "Read_Local_Extended_Features", params: &[
        ("Page_Number", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x04, ocf: 0x0005, name: "Read_Buffer_Size", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x0009, name: "Read_BD_ADDR", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x000A, name: "Read_Data_Block_Size", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x000B, name: "Read_Local_Supported_Codecs_V1", params: &[] },
    HciCmdDef { ogf: 0x04, ocf: 0x000D, name: "Read_Local_Supported_Codecs_V2", params: &[] },

    // OGF=0x05 Status Parameters
    HciCmdDef { ogf: 0x05, ocf: 0x0001, name: "Read_Failed_Contact_Counter", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x05, ocf: 0x0002, name: "Reset_Failed_Contact_Counter", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x05, ocf: 0x0003, name: "Read_Link_Quality", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x05, ocf: 0x0005, name: "Read_RSSI", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x05, ocf: 0x0006, name: "Read_AFH_Channel_Map", params: &[
        ("Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x05, ocf: 0x0007, name: "Read_Clock", params: &[
        ("Handle", ParamType::U16),
        ("Which_Clock", ParamType::U8),
    ]},

    // OGF=0x06 Testing
    HciCmdDef { ogf: 0x06, ocf: 0x0001, name: "Read_Loopback_Mode", params: &[] },
    HciCmdDef { ogf: 0x06, ocf: 0x0002, name: "Write_Loopback_Mode", params: &[
        ("Loopback_Mode", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x06, ocf: 0x0003, name: "Enable_Device_Under_Test_Mode", params: &[] },

    // OGF=0x08 LE Controller
    HciCmdDef { ogf: 0x08, ocf: 0x0001, name: "LE_Set_Event_Mask", params: &[
        ("LE_Event_Mask", ParamType::Bytes(8)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0002, name: "LE_Read_Buffer_Size_V1", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0003, name: "LE_Read_Local_Supported_Features", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0005, name: "LE_Set_Random_Address", params: &[
        ("Random_Address", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0006, name: "LE_Set_Advertising_Parameters", params: &[
        ("Advertising_Interval_Min", ParamType::U16),
        ("Advertising_Interval_Max", ParamType::U16),
        ("Advertising_Type", ParamType::U8),
        ("Own_Address_Type", ParamType::U8),
        ("Peer_Address_Type", ParamType::U8),
        ("Peer_Address", ParamType::BdAddr),
        ("Advertising_Channel_Map", ParamType::U8),
        ("Advertising_Filter_Policy", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0007, name: "LE_Read_Advertising_Channel_Tx_Power", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0008, name: "LE_Set_Advertising_Data", params: &[
        ("Advertising_Data_Length", ParamType::U8),
        ("Advertising_Data", ParamType::Bytes(31)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0009, name: "LE_Set_Scan_Response_Data", params: &[
        ("Scan_Response_Data_Length", ParamType::U8),
        ("Scan_Response_Data", ParamType::Bytes(31)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x000A, name: "LE_Set_Advertising_Enable", params: &[
        ("Advertising_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x000B, name: "LE_Set_Scan_Parameters", params: &[
        ("LE_Scan_Type", ParamType::U8),
        ("LE_Scan_Interval", ParamType::U16),
        ("LE_Scan_Window", ParamType::U16),
        ("Own_Address_Type", ParamType::U8),
        ("Scanning_Filter_Policy", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x000C, name: "LE_Set_Scan_Enable", params: &[
        ("LE_Scan_Enable", ParamType::U8),
        ("Filter_Duplicates", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x000D, name: "LE_Create_Connection", params: &[
        ("LE_Scan_Interval", ParamType::U16),
        ("LE_Scan_Window", ParamType::U16),
        ("Initiator_Filter_Policy", ParamType::U8),
        ("Peer_Address_Type", ParamType::U8),
        ("Peer_Address", ParamType::BdAddr),
        ("Own_Address_Type", ParamType::U8),
        ("Conn_Interval_Min", ParamType::U16),
        ("Conn_Interval_Max", ParamType::U16),
        ("Conn_Latency", ParamType::U16),
        ("Supervision_Timeout", ParamType::U16),
        ("Min_CE_Length", ParamType::U16),
        ("Max_CE_Length", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x000E, name: "LE_Create_Connection_Cancel", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x000F, name: "LE_Read_Filter_Accept_List_Size", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0010, name: "LE_Clear_Filter_Accept_List", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0011, name: "LE_Add_Device_To_Filter_Accept_List", params: &[
        ("Address_Type", ParamType::U8),
        ("Address", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0012, name: "LE_Remove_Device_From_Filter_Accept_List", params: &[
        ("Address_Type", ParamType::U8),
        ("Address", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0014, name: "LE_Read_Remote_Features", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0016, name: "LE_Encrypt", params: &[
        ("Key", ParamType::Bytes(16)),
        ("Plaintext_Data", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0017, name: "LE_Rand", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0018, name: "LE_Enable_Encryption", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Random_Number", ParamType::Bytes(8)),
        ("Encrypted_Diversifier", ParamType::U16),
        ("Long_Term_Key", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0019, name: "LE_Long_Term_Key_Request_Reply", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Long_Term_Key", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x001A, name: "LE_Long_Term_Key_Request_Negative_Reply", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x001B, name: "LE_Read_Supported_States", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x001C, name: "LE_Receiver_Test_V1", params: &[
        ("RX_Channel", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x001D, name: "LE_Transmitter_Test_V1", params: &[
        ("TX_Channel", ParamType::U8),
        ("Test_Data_Length", ParamType::U8),
        ("Packet_Payload", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x001E, name: "LE_Test_End", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x001F, name: "LE_Remote_Connection_Parameter_Request_Reply", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Interval_Min", ParamType::U16),
        ("Interval_Max", ParamType::U16),
        ("Latency", ParamType::U16),
        ("Timeout", ParamType::U16),
        ("Min_CE_Length", ParamType::U16),
        ("Max_CE_Length", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0020, name: "LE_Remote_Connection_Parameter_Request_Negative_Reply", params: &[
        ("Connection_Handle", ParamType::U16),
        ("Reason", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0021, name: "LE_Set_Data_Length", params: &[
        ("Connection_Handle", ParamType::U16),
        ("TX_Octets", ParamType::U16),
        ("TX_Time", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0022, name: "LE_Read_Suggested_Default_Data_Length", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0023, name: "LE_Write_Suggested_Default_Data_Length", params: &[
        ("Suggested_Max_TX_Octets", ParamType::U16),
        ("Suggested_Max_TX_Time", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0024, name: "LE_Read_Local_P256_Public_Key", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0025, name: "LE_Generate_DHKey_V1", params: &[
        ("Remote_P256_Public_Key", ParamType::Bytes(64)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0026, name: "LE_Add_Device_To_Resolving_List", params: &[
        ("Peer_Identity_Address_Type", ParamType::U8),
        ("Peer_Identity_Address", ParamType::BdAddr),
        ("Peer_IRK", ParamType::Bytes(16)),
        ("Local_IRK", ParamType::Bytes(16)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0029, name: "LE_Read_Peer_Resolvable_Address", params: &[
        ("Peer_Identity_Address_Type", ParamType::U8),
        ("Peer_Identity_Address", ParamType::BdAddr),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x002D, name: "LE_Set_Address_Resolution_Enable", params: &[
        ("Address_Resolution_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x002F, name: "LE_Read_Maximum_Data_Length", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0030, name: "LE_Read_PHY", params: &[
        ("Connection_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0031, name: "LE_Set_Default_PHY", params: &[
        ("ALL_PHYs", ParamType::U8),
        ("TX_PHYs", ParamType::U8),
        ("RX_PHYs", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0032, name: "LE_Set_PHY", params: &[
        ("Connection_Handle", ParamType::U16),
        ("ALL_PHYs", ParamType::U8),
        ("TX_PHYs", ParamType::U8),
        ("RX_PHYs", ParamType::U8),
        ("PHY_Options", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0033, name: "LE_Receiver_Test_V2", params: &[
        ("RX_Channel", ParamType::U8),
        ("PHY", ParamType::U8),
        ("Modulation_Index", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0034, name: "LE_Transmitter_Test_V2", params: &[
        ("TX_Channel", ParamType::U8),
        ("Test_Data_Length", ParamType::U8),
        ("Packet_Payload", ParamType::U8),
        ("PHY", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0035, name: "LE_Set_Extended_Advertising_Parameters", params: &[
        ("Advertising_Handle", ParamType::U8),
        ("Advertising_Event_Properties", ParamType::U16),
        ("Primary_Advertising_Interval_Min", ParamType::U24),
        ("Primary_Advertising_Interval_Max", ParamType::U24),
        ("Primary_Advertising_Channel_Map", ParamType::U8),
        ("Own_Address_Type", ParamType::U8),
        ("Peer_Address_Type", ParamType::U8),
        ("Peer_Address", ParamType::BdAddr),
        ("Advertising_Filter_Policy", ParamType::U8),
        ("Advertising_TX_Power", ParamType::U8),
        ("Primary_Advertising_PHY", ParamType::U8),
        ("Secondary_Advertising_Max_Skip", ParamType::U8),
        ("Secondary_Advertising_PHY", ParamType::U8),
        ("Advertising_SID", ParamType::U8),
        ("Scan_Request_Notification_Enable", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0036, name: "LE_Read_Maximum_Advertising_Data_Length", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0037, name: "LE_Set_Extended_Advertising_Data", params: &[
        ("Advertising_Handle", ParamType::U8),
        ("Operation", ParamType::U8),
        ("Fragment_Preference", ParamType::U8),
        ("Advertising_Data_Length", ParamType::U8),
        ("Advertising_Data", ParamType::Bytes(0)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0038, name: "LE_Set_Extended_Scan_Response_Data", params: &[
        ("Advertising_Handle", ParamType::U8),
        ("Operation", ParamType::U8),
        ("Fragment_Preference", ParamType::U8),
        ("Scan_Response_Data_Length", ParamType::U8),
        ("Scan_Response_Data", ParamType::Bytes(0)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0039, name: "LE_Read_Number_of_Supported_Advertising_Sets", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x003A, name: "LE_Remove_Advertising_Set", params: &[
        ("Advertising_Handle", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x003B, name: "LE_Clear_Advertising_Sets", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x003C, name: "LE_Set_Periodic_Advertising_Parameters", params: &[
        ("Advertising_Handle", ParamType::U8),
        ("Periodic_Advertising_Interval_Min", ParamType::U16),
        ("Periodic_Advertising_Interval_Max", ParamType::U16),
        ("Periodic_Advertising_Properties", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x003D, name: "LE_Set_Periodic_Advertising_Data", params: &[
        ("Advertising_Handle", ParamType::U8),
        ("Operation", ParamType::U8),
        ("Advertising_Data_Length", ParamType::U8),
        ("Advertising_Data", ParamType::Bytes(0)),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x003E, name: "LE_Set_Extended_Advertising_Enable", params: &[
        ("Enable", ParamType::U8),
        ("Num_Sets", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x003F, name: "LE_Set_Periodic_Advertising_Enable", params: &[
        ("Enable", ParamType::U8),
        ("Advertising_Handle", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0044, name: "LE_Periodic_Advertising_Create_Sync", params: &[
        ("Options", ParamType::U8),
        ("Advertising_SID", ParamType::U8),
        ("Advertiser_Address_Type", ParamType::U8),
        ("Advertiser_Address", ParamType::BdAddr),
        ("Skip", ParamType::U16),
        ("Sync_Timeout", ParamType::U16),
        ("Sync_CTE_Type", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0045, name: "LE_Periodic_Advertising_Create_Sync_Cancel", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0046, name: "LE_Periodic_Advertising_Terminate_Sync", params: &[
        ("Sync_Handle", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0047, name: "LE_Add_Device_To_Periodic_Advertiser_List", params: &[
        ("Advertiser_Address_Type", ParamType::U8),
        ("Advertiser_Address", ParamType::BdAddr),
        ("Advertising_SID", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0048, name: "LE_Remove_Device_From_Periodic_Advertiser_List", params: &[
        ("Advertiser_Address_Type", ParamType::U8),
        ("Advertiser_Address", ParamType::BdAddr),
        ("Advertising_SID", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0049, name: "LE_Clear_Periodic_Advertiser_List", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x004A, name: "LE_Read_Periodic_Advertiser_List_Size", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x004B, name: "LE_Read_Transmit_Power", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x004C, name: "LE_Read_RF_Path_Compensation", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x004D, name: "LE_Write_RF_Path_Compensation", params: &[
        ("TX_Path_Compensation", ParamType::U16),
        ("RX_Path_Compensation", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x004E, name: "LE_Set_Privacy_Mode", params: &[
        ("Peer_Identity_Address_Type", ParamType::U8),
        ("Peer_Identity_Address", ParamType::BdAddr),
        ("Privacy_Mode", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0041, name: "LE_Set_Extended_Scan_Parameters", params: &[
        ("Own_Address_Type", ParamType::U8),
        ("Scanning_Filter_Policy", ParamType::U8),
        ("Scanning_PHYs", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0042, name: "LE_Set_Extended_Scan_Enable", params: &[
        ("Enable", ParamType::U8),
        ("Filter_Duplicates", ParamType::U8),
        ("Duration", ParamType::U16),
        ("Period", ParamType::U16),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0043, name: "LE_Extended_Create_Connection", params: &[
        ("Initiator_Filter_Policy", ParamType::U8),
        ("Own_Address_Type", ParamType::U8),
        ("Peer_Address_Type", ParamType::U8),
        ("Peer_Address", ParamType::BdAddr),
        ("Initiating_PHYs", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0058, name: "LE_Set_CIG_Parameters", params: &[
        ("CIG_ID", ParamType::U8),
        ("SDU_Interval_M_To_S", ParamType::U24),
        ("SDU_Interval_S_To_M", ParamType::U24),
        ("FT_M_To_S", ParamType::U8),
        ("FT_S_To_M", ParamType::U8),
        ("ISO_Interval", ParamType::U16),
        ("CIG_SCA", ParamType::U8),
        ("Packing", ParamType::U8),
        ("Framing", ParamType::U8),
        ("Num_CIS", ParamType::U8),
    ]},
    HciCmdDef { ogf: 0x08, ocf: 0x0062, name: "LE_Read_Buffer_Size_V2", params: &[] },
    HciCmdDef { ogf: 0x08, ocf: 0x0068, name: "LE_Extended_Create_Connection_V2", params: &[
        ("Adv_Handle", ParamType::U8),
        ("Subevent", ParamType::U8),
        ("Initiator_Filter_Policy", ParamType::U8),
        ("Own_Address_Type", ParamType::U8),
        ("Peer_Address_Type", ParamType::U8),
        ("Peer_Address", ParamType::BdAddr),
        ("Initiating_PHYs", ParamType::U8),
    ]},
];
