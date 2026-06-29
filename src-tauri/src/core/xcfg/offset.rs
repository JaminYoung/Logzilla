use super::types::{ConfigItem, EntryType};
use log::info;

pub fn is_bitfield(item: &ConfigItem) -> bool {
    match item.entry_type {
        EntryType::CHK => true,
        EntryType::UBT => true,
        EntryType::LSV => (item.val_type & 0xFF) == 0x20,
        _ => false,
    }
}

pub fn bitfield_width(item: &ConfigItem) -> usize {
    match item.entry_type {
        EntryType::CHK => 1,
        EntryType::UBT => {
            if item.bit_width > 0 {
                item.bit_width as usize
            } else {
                1
            }
        }
        EntryType::LSV => ((item.val_type >> 8) & 0xFF) as usize,
        _ => 0,
    }
}

pub fn non_bitfield_size(item: &ConfigItem) -> usize {
    match item.entry_type {
        EntryType::U08 | EntryType::S08 | EntryType::LST => 1,
        EntryType::U16 => 2,
        EntryType::LSV => {
            let vt = item.val_type & 0xFF;
            if vt == 0x10 {
                2
            } else {
                4
            }
        }
        EntryType::MAC => 6,
        EntryType::TXT => {
            if item.str_length > 0 {
                item.str_length as usize
            } else {
                32
            }
        }
        _ => 0,
    }
}

pub fn assign_offsets(items: &mut [ConfigItem]) {
    let mut byte_off: i32 = 0;
    let mut bit_off: u8 = 0;
    let mut bits_used: u8 = 0;
    let mut in_bitfield_group = false;

    assign_offsets_recursive(items, &mut byte_off, &mut bit_off, &mut bits_used, &mut in_bitfield_group);

    // Final alignment if ended in bitfield group
    if in_bitfield_group {
        byte_off += 4;
    }
    
    info!("Offset assignment complete: final byte_off={}", byte_off);
}

fn assign_offsets_recursive(
    items: &mut [ConfigItem],
    byte_off: &mut i32,
    bit_off: &mut u8,
    bits_used: &mut u8,
    in_bitfield_group: &mut bool,
) {
    for item in items.iter_mut() {
        match item.entry_type {
            EntryType::SUB => {
                assign_offsets_recursive(
                    &mut item.children,
                    byte_off,
                    bit_off,
                    bits_used,
                    in_bitfield_group,
                );
            }
            EntryType::LVL => {
                // LVL has no data, just UI control
            }
            _ => {
                if item.var_name.is_empty() {
                    continue;
                }

                if is_bitfield(item) {
                    let bw = bitfield_width(item) as u8;

                    if !*in_bitfield_group {
                        *bit_off = 0;
                        *bits_used = 0;
                        *in_bitfield_group = true;
                    }

                    // Check if current bitfield group has enough space
                    if *bits_used + bw > 32 {
                        *byte_off += 4;
                        *bit_off = 0;
                        *bits_used = 0;
                    }

                    item.offset = *byte_off;
                    item.bit_offset = *bit_off;
                    item.bit_width = bw;
                    item.size = 0;

                    *bit_off += bw;
                    *bits_used += bw;
                } else {
                    // Non-bitfield: align to next 4-byte boundary if in bitfield group
                    if *in_bitfield_group {
                        *byte_off += 4;
                        *bit_off = 0;
                        *bits_used = 0;
                        *in_bitfield_group = false;
                    }

                    let sz = non_bitfield_size(item) as i32;
                    item.offset = *byte_off;
                    item.bit_offset = 0;
                    item.bit_width = 0;
                    item.size = sz as u8;

                    *byte_off += sz;
                }
            }
        }
    }
}
