//! Helper functions for CRC8 checksum validation

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

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    WrongCrc,
    WrongBufferSize,
}

/// Iterate over the provided buffer and validate the CRC8 checksums.
///
/// The buffer must be in the form of `[d0, d1, crc01, d2, d3, crc23, ...]` where every third byte
/// is the checksum byte of the previous two bytes
/// If any checksum is wrong, return `Error::WrongCrc`.
///
/// This method will consider every third byte a checksum byte. If the buffer size is not a
/// multiple of 3, then it will return `Error::WrongBufferSize`
pub fn validate(buf: &[u8]) -> Result<(), Error> {
    if buf.len() % 3 != 0 {
        return Err(Error::WrongBufferSize);
    }
    for chunk in buf.chunks(3) {
        if calculate(&[chunk[0], chunk[1]]) != chunk[2] {
            return Err(Error::WrongCrc);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::crc8;
    use crate::crc8::Error;

    /// Test the crc function against the test value provided in the SHTC3 datasheet (section
    /// 5.10).
    #[test]
    fn crc8_test_value() {
        assert_eq!(crc8::calculate(&[0x00]), 0xac);
        assert_eq!(crc8::calculate(&[0xbe, 0xef]), 0x92);
    }

    #[test]
    fn crc8_validate_empty_ok() {
        assert!(crc8::validate(&[]).is_ok());
    }

    #[test]
    fn crc8_validate_not_enough_data() {
        assert_eq!(Err(Error::WrongBufferSize), crc8::validate(&[0xbe]));
    }

    #[test]
    fn crc8_validate() {
        // Valid CRC
        assert!(crc8::validate(&[0xbe, 0xef, 0x92]).is_ok());

        // Invalid CRC
        assert_eq!(crc8::validate(&[0xbe, 0xef, 0x91]), Err(Error::WrongCrc));
    }
}
