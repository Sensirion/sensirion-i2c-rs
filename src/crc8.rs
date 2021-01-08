//! Helper functions for CRC8 checksum validation

/// Errors which can happen in the crc8 module
#[derive(Debug, PartialEq)]
pub enum CrcError {
    /// CRC validation failed
    CrcError,
    /// Invalid length (not a multiple of 3)
    InvalidBufferSize,
}

impl<W, R> From<CrcError> for crate::i2c::Error<W, R>
where
    W: embedded_hal::blocking::i2c::Write,
    R: embedded_hal::blocking::i2c::Read,
{
    fn from(e: CrcError) -> Self {
        match e {
            CrcError::CrcError => crate::i2c::Error::CrcError,
            CrcError::InvalidBufferSize => crate::i2c::Error::InvalidBufferSize,
        }
    }
}

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

/// Iterate over the provided buffer and validate the CRC8 checksums.
///
/// The buffer must be in the form of `[d0, d1, crc01, d2, d3, crc23, ...]` where every third byte
/// is the checksum byte of the previous two bytes
/// If the checksum is wrong, return `Err`.
pub fn validate(buf: &[u8]) -> Result<(), CrcError> {
    if buf.len() % 3 != 0 {
        return Err(CrcError::InvalidBufferSize);
    }
    for chunk in buf.chunks_exact(3) {
        if calculate(&[chunk[0], chunk[1]]) != chunk[2] {
            return Err(CrcError::CrcError);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::crc8;

    #[test]
    fn crc8_validate_empty_valid() {
        let buffer: [u8; 0] = [];
        crc8::validate(&buffer).unwrap()
    }

    #[test]
    #[should_panic]
    fn crc8_validate_one_invalid() {
        let buffer: [u8; 1] = [3];
        crc8::validate(&buffer).unwrap()
    }

    /// Test the crc function against the test value provided in the SHTC3 datasheet (section
    /// 5.10).
    #[test]
    fn crc8_test_value() {
        assert_eq!(crc8::calculate(&[0x00]), 0xac);
        assert_eq!(crc8::calculate(&[0xbe, 0xef]), 0x92);
    }

    #[test]
    fn crc8_validate_valid() {
        let data = [0xbeu8, 0xef, 0x92];
        assert!(crc8::validate(&data).is_ok());
    }

    #[test]
    fn crc8_validate_invalid() {
        let buffer: [u8; 3] = [0xbe, 0xef, 0x91];
        assert_eq!(crc8::validate(&buffer), Err(crc8::CrcError::CrcError));
    }

    #[test]
    fn crc8_validate() {
        // Valid CRC
        crc8::validate(&[0xbe, 0xef, 0x92]).unwrap();

        // Invalid CRC
        assert_eq!(
            crc8::validate(&[0xbe, 0xef, 0x91]),
            Err(crc8::CrcError::CrcError)
        );
    }
}
