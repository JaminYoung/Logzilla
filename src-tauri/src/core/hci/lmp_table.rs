use super::types::{LmpDef, ParamType};

pub fn lookup(opcode: u8) -> Option<&'static LmpDef> {
    TABLE.iter().find(|l| l.opcode == opcode)
}

pub fn lookup_ext(opcode: u8) -> Option<&'static LmpDef> {
    EXT_TABLE.iter().find(|l| l.opcode == opcode)
}

static TABLE: &[LmpDef] = &[
    LmpDef {
        opcode: 1,
        name: "LMP_name_req",
        params: &[],
    },
    LmpDef {
        opcode: 2,
        name: "LMP_name_res",
        params: &[
            ("offset", ParamType::U8),
            ("name_length", ParamType::U8),
            ("name_fragment", ParamType::Bytes(14)),
        ],
    },
    LmpDef {
        opcode: 3,
        name: "LMP_accepted",
        params: &[("opcode", ParamType::U8)],
    },
    LmpDef {
        opcode: 4,
        name: "LMP_not_accepted",
        params: &[("opcode", ParamType::U8), ("error_code", ParamType::U8)],
    },
    LmpDef {
        opcode: 5,
        name: "LMP_clkoffset_req",
        params: &[],
    },
    LmpDef {
        opcode: 6,
        name: "LMP_clkoffset_res",
        params: &[("clock_offset", ParamType::U16)],
    },
    LmpDef {
        opcode: 7,
        name: "LMP_detach",
        params: &[("error_code", ParamType::U8)],
    },
    LmpDef {
        opcode: 8,
        name: "LMP_in_rand",
        params: &[("random_number", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 9,
        name: "LMP_comb_key",
        params: &[("random_number", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 10,
        name: "LMP_unit_key",
        params: &[("key", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 11,
        name: "LMP_au_rand",
        params: &[("random_number", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 12,
        name: "LMP_sres",
        params: &[("authentication_response", ParamType::Bytes(4))],
    },
    LmpDef {
        opcode: 13,
        name: "LMP_temp_rand",
        params: &[("random_number", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 14,
        name: "LMP_temp_key",
        params: &[("key", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 15,
        name: "LMP_encryption_mode_req",
        params: &[("encryption_mode", ParamType::U8)],
    },
    LmpDef {
        opcode: 16,
        name: "LMP_encryption_key_size_req",
        params: &[("key_size", ParamType::U8)],
    },
    LmpDef {
        opcode: 17,
        name: "LMP_start_encryption_req",
        params: &[("random_number", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 18,
        name: "LMP_stop_encryption_req",
        params: &[],
    },
    LmpDef {
        opcode: 19,
        name: "LMP_switch_req",
        params: &[("switch_instant", ParamType::U16)],
    },
    LmpDef {
        opcode: 20,
        name: "LMP_hold",
        params: &[
            ("hold_time", ParamType::U16),
            ("hold_instant", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 21,
        name: "LMP_hold_req",
        params: &[
            ("hold_time", ParamType::U16),
            ("hold_instant", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 23,
        name: "LMP_sniff_req",
        params: &[
            ("timing_control_flags", ParamType::U8),
            ("D_sniff", ParamType::U16),
            ("T_sniff", ParamType::U16),
            ("sniff_attempt", ParamType::U16),
            ("sniff_timeout", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 24,
        name: "LMP_unsniff_req",
        params: &[],
    },
    LmpDef {
        opcode: 25,
        name: "LMP_park_req",
        params: &[
            ("timing_control_flags", ParamType::U8),
            ("D", ParamType::U16),
            ("T", ParamType::U16),
            ("delta_b", ParamType::U8),
            ("d_b", ParamType::U8),
            ("T_b", ParamType::U8),
            ("M_b", ParamType::U8),
            ("N_b", ParamType::U8),
            ("delta_d", ParamType::U8),
            ("d_d", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 26,
        name: "LMP_set_broadcast_scan_window",
        params: &[
            ("timing_control_flags", ParamType::U8),
            ("D_b", ParamType::U16),
            ("T_b", ParamType::U8),
            ("M_b", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 27,
        name: "LMP_modify_beacon",
        params: &[
            ("timing_control_flags", ParamType::U8),
            ("D", ParamType::U16),
            ("T", ParamType::U16),
            ("delta_b", ParamType::U8),
            ("d_b", ParamType::U8),
            ("T_b", ParamType::U8),
            ("M_b", ParamType::U8),
            ("N_b", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 28,
        name: "LMP_unpark_BD_ADDR_req",
        params: &[("AM_ADDR", ParamType::U8), ("BD_ADDR", ParamType::BdAddr)],
    },
    LmpDef {
        opcode: 29,
        name: "LMP_unpark_PM_ADDR_req",
        params: &[("AM_ADDR", ParamType::U8), ("PM_ADDR", ParamType::U8)],
    },
    LmpDef {
        opcode: 30,
        name: "LMP_incr_power_req",
        params: &[],
    },
    LmpDef {
        opcode: 31,
        name: "LMP_decr_power_req",
        params: &[],
    },
    LmpDef {
        opcode: 32,
        name: "LMP_max_power",
        params: &[],
    },
    LmpDef {
        opcode: 33,
        name: "LMP_min_power",
        params: &[],
    },
    LmpDef {
        opcode: 34,
        name: "LMP_auto_rate",
        params: &[],
    },
    LmpDef {
        opcode: 35,
        name: "LMP_preferred_rate",
        params: &[("data_rate", ParamType::U8)],
    },
    LmpDef {
        opcode: 36,
        name: "LMP_version_req",
        params: &[
            ("vers_nr", ParamType::U8),
            ("comp_id", ParamType::U16),
            ("subvers_nr", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 37,
        name: "LMP_features_req_ext",
        params: &[
            ("features_page", ParamType::U8),
            ("max_supported_page", ParamType::U8),
            ("extended_features", ParamType::Bytes(8)),
        ],
    },
    LmpDef {
        opcode: 38,
        name: "LMP_features_res_ext",
        params: &[
            ("features_page", ParamType::U8),
            ("max_supported_page", ParamType::U8),
            ("extended_features", ParamType::Bytes(8)),
        ],
    },
    LmpDef {
        opcode: 39,
        name: "LMP_features_req",
        params: &[("features", ParamType::Bytes(8))],
    },
    LmpDef {
        opcode: 40,
        name: "LMP_features_res",
        params: &[("features", ParamType::Bytes(8))],
    },
    LmpDef {
        opcode: 41,
        name: "LMP_quality_of_service",
        params: &[("poll_interval", ParamType::U16), ("Nbc", ParamType::U8)],
    },
    LmpDef {
        opcode: 42,
        name: "LMP_quality_of_service_req",
        params: &[("poll_interval", ParamType::U16), ("Nbc", ParamType::U8)],
    },
    LmpDef {
        opcode: 43,
        name: "LMP_SCO_link_req",
        params: &[
            ("SCO_handle", ParamType::U8),
            ("Dsco", ParamType::U8),
            ("Tsco", ParamType::U8),
            ("SCO_packet_type", ParamType::U8),
            ("air_mode", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 44,
        name: "LMP_remove_SCO_link_req",
        params: &[("SCO_handle", ParamType::U8), ("error_code", ParamType::U8)],
    },
    LmpDef {
        opcode: 45,
        name: "LMP_max_slot",
        params: &[("max_slots", ParamType::U8)],
    },
    LmpDef {
        opcode: 46,
        name: "LMP_max_slot_req",
        params: &[("max_slots", ParamType::U8)],
    },
    LmpDef {
        opcode: 47,
        name: "LMP_timing_accuracy_req",
        params: &[],
    },
    LmpDef {
        opcode: 48,
        name: "LMP_timing_accuracy_res",
        params: &[("drift", ParamType::U8), ("jitter", ParamType::U8)],
    },
    LmpDef {
        opcode: 49,
        name: "LMP_setup_complete",
        params: &[],
    },
    LmpDef {
        opcode: 50,
        name: "LMP_use_semi_permanent_key",
        params: &[],
    },
    LmpDef {
        opcode: 51,
        name: "LMP_host_connection_req",
        params: &[],
    },
    LmpDef {
        opcode: 52,
        name: "LMP_slot_offset",
        params: &[
            ("slot_offset", ParamType::U16),
            ("BD_ADDR", ParamType::BdAddr),
        ],
    },
    LmpDef {
        opcode: 53,
        name: "LMP_page_mode_req",
        params: &[
            ("paging_scheme", ParamType::U8),
            ("paging_scheme_settings", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 54,
        name: "LMP_page_scan_mode_req",
        params: &[
            ("paging_scheme", ParamType::U8),
            ("paging_scheme_settings", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 55,
        name: "LMP_supervision_timeout",
        params: &[("supervision_timeout", ParamType::U16)],
    },
    LmpDef {
        opcode: 56,
        name: "LMP_test_activate",
        params: &[],
    },
    LmpDef {
        opcode: 57,
        name: "LMP_test_control",
        params: &[
            ("test_scenario", ParamType::U8),
            ("hopping_mode", ParamType::U8),
            ("TX_frequency", ParamType::U8),
            ("RX_frequency", ParamType::U8),
            ("power_control_mode", ParamType::U8),
            ("poll_period", ParamType::U8),
            ("packet_type", ParamType::U8),
            ("length_of_test_data", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 58,
        name: "LMP_encryption_key_size_mask_req",
        params: &[],
    },
    LmpDef {
        opcode: 59,
        name: "LMP_encryption_key_size_mask_res",
        params: &[("key_size_mask", ParamType::U16)],
    },
    LmpDef {
        opcode: 60,
        name: "LMP_set_AFH",
        params: &[
            ("AFH_instant", ParamType::U32),
            ("AFH_mode", ParamType::U8),
            ("AFH_channel_map", ParamType::Bytes(10)),
        ],
    },
    LmpDef {
        opcode: 61,
        name: "LMP_encapsulated_header",
        params: &[
            ("major_type", ParamType::U8),
            ("minor_type", ParamType::U8),
            ("payload_length", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 62,
        name: "LMP_encapsulated_payload",
        params: &[("encapsulated_data", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 63,
        name: "LMP_simple_pairing_confirm",
        params: &[("commitment_value", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 64,
        name: "LMP_simple_pairing_number",
        params: &[("nonce_value", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 65,
        name: "LMP_DHkey_check",
        params: &[("check_value", ParamType::Bytes(16))],
    },
    LmpDef {
        opcode: 66,
        name: "LMP_pause_encryption_aes_req",
        params: &[],
    },
    LmpDef {
        opcode: 127,
        name: "LMP_escaped",
        params: &[("extended_opcode", ParamType::U8)],
    },
];

static EXT_TABLE: &[LmpDef] = &[
    LmpDef {
        opcode: 1,
        name: "LMP_accepted_ext",
        params: &[("escaped_opcode", ParamType::U8)],
    },
    LmpDef {
        opcode: 2,
        name: "LMP_not_accepted_ext",
        params: &[
            ("escaped_opcode", ParamType::U8),
            ("error_code", ParamType::U8),
        ],
    },
    LmpDef {
        opcode: 3,
        name: "LMP_packet_type_table_req",
        params: &[("packet_type_table", ParamType::U8)],
    },
    LmpDef {
        opcode: 4,
        name: "LMP_eSCO_link_req",
        params: &[("parameters", ParamType::Bytes(0))],
    },
    LmpDef {
        opcode: 5,
        name: "LMP_remove_eSCO_link_req",
        params: &[("eSCO_handle", ParamType::U8), ("reason", ParamType::U8)],
    },
    LmpDef {
        opcode: 6,
        name: "LMP_channel_classification_req",
        params: &[
            ("reporting_enable", ParamType::U8),
            ("min_interval", ParamType::U16),
            ("max_interval", ParamType::U16),
        ],
    },
    LmpDef {
        opcode: 7,
        name: "LMP_channel_classification",
        params: &[
            ("AFH_instant", ParamType::U32),
            ("AFH_channel_classification", ParamType::Bytes(10)),
        ],
    },
    LmpDef {
        opcode: 8,
        name: "LMP_sniff_subrating_req",
        params: &[("parameters", ParamType::Bytes(0))],
    },
    LmpDef {
        opcode: 9,
        name: "LMP_sniff_subrating_res",
        params: &[("parameters", ParamType::Bytes(0))],
    },
    LmpDef {
        opcode: 10,
        name: "LMP_pause_encryption_req",
        params: &[],
    },
    LmpDef {
        opcode: 11,
        name: "LMP_resume_encryption_req",
        params: &[],
    },
    LmpDef {
        opcode: 12,
        name: "LMP_IO_capability_req",
        params: &[("parameters", ParamType::Bytes(0))],
    },
    LmpDef {
        opcode: 13,
        name: "LMP_IO_capability_res",
        params: &[("parameters", ParamType::Bytes(0))],
    },
    LmpDef {
        opcode: 14,
        name: "LMP_numeric_comparison_failed",
        params: &[],
    },
    LmpDef {
        opcode: 15,
        name: "LMP_passkey_failed",
        params: &[],
    },
    LmpDef {
        opcode: 16,
        name: "LMP_oob_failed",
        params: &[],
    },
    LmpDef {
        opcode: 17,
        name: "LMP_keypress_notification",
        params: &[("notification_type", ParamType::U8)],
    },
    LmpDef {
        opcode: 18,
        name: "LMP_power_control_req",
        params: &[("power_adjustment_request", ParamType::U8)],
    },
    LmpDef {
        opcode: 19,
        name: "LMP_power_control_res",
        params: &[("power_adjustment_response", ParamType::U8)],
    },
];
