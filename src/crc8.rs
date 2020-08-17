/// Calculate the CRC8 checksum.
pub fn calculate(data: &[u8]) -> u8 {
    const CRC8_POLYNOMIAL: u8 = 0x31;
    let mut crc: u8 = 0xff;
    for byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) > 0 {
                crc = (crc << 1) ^ CRC8_POLYNOMIAL;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Iterate over the provided buffer and validate the CRC8 checksum.
///
/// If the checksum is wrong, return `Err`.
///
/// Note: This method will consider every third byte a checksum byte. If the buffer size is not a
/// multiple of 3, then not all data will be validated.
pub fn validate(buf: &[u8]) -> Result<(), ()> {
    debug_assert!(buf.len() % 3 == 0, "Buffer must be a multiple of 3");
    for chunk in buf.chunks(3) {
        if chunk.len() == 3 && calculate(&[chunk[0], chunk[1]]) != chunk[2] {
            return Err(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::crc8;

    /// Test the crc function against the test value provided in the SHTC3 datasheet (section
    /// 5.10).
    #[test]
    fn crc8_test_value() {
        assert_eq!(crc8::calculate(&[0x00]), 0xac);
        assert_eq!(crc8::calculate(&[0xbe, 0xef]), 0x92);
    }
}
