pub fn recalculate_checksum(dcf_data: &mut [u8], info_offset: usize, info_len: usize) {
    let info_start = info_offset + 16;
    let mut checksum: u8 = 0;

    for i in info_start..info_start + info_len {
        checksum = checksum.wrapping_add(dcf_data[i]);
    }

    // Two's complement
    checksum = (!checksum).wrapping_add(1);

    // Checksum is stored at info_offset + 15
    dcf_data[info_offset + 15] = checksum;
}

pub fn verify_checksum(dcf_data: &[u8], info_offset: usize, info_len: usize) -> bool {
    let info_start = info_offset + 16;
    let mut checksum: u8 = 0;

    for i in info_start..info_start + info_len {
        checksum = checksum.wrapping_add(dcf_data[i]);
    }

    // Add the stored checksum byte - should result in 0
    checksum = checksum.wrapping_add(dcf_data[info_offset + 15]);

    checksum == 0
}
