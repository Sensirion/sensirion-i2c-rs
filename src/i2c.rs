//! Helper functions for the u16 word based I²C communication.

use crate::crc8;
use embedded_hal::blocking::i2c;

/// All possible errors in this crate
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error<I2cWrite: i2c::Write, I2cRead: i2c::Read> {
    I2cWrite(I2cWrite::Error),
    I2cRead(I2cRead::Error),
    Crc,
}

/// Write an u16 command to the I²C bus.
pub fn write_command<I2cWrite: i2c::Write>(
    i2c: &mut I2cWrite,
    addr: u8,
    command: u16,
) -> Result<(), I2cWrite::Error> {
    i2c.write(addr, &command.to_be_bytes())
}

/// Read data into the provided buffer and validate the CRC8 checksum.
///
/// If the checksum is wrong, return `Error::Crc`.
///
/// # Panics
///
/// This method will consider every third byte a checksum byte. If the buffer size is not a
/// multiple of 3, then it will panic.
pub fn read_words_with_crc<I2c: i2c::Read + i2c::Write>(
    i2c: &mut I2c,
    addr: u8,
    data: &mut [u8],
) -> Result<(), Error<I2c, I2c>> {
    assert!(
        data.len() % 3 == 0,
        "Buffer must hold a multiple of 3 bytes"
    );
    i2c.read(addr, data).map_err(Error::I2cRead)?;
    crc8::validate(data).map_err(|_| Error::Crc)
}

#[cfg(test)]
mod tests {
    use crate::i2c;

    use embedded_hal_mock as hal;
    use hal::i2c::{Mock as I2cMock, Transaction};

    #[test]
    fn read_words_with_crc() {
        let mut buf = [0; 3];

        // Valid CRC
        let expectations = [Transaction::read(0x58, vec![0xBE, 0xEF, 0x92])];
        let mut mock = I2cMock::new(&expectations);
        i2c::read_words_with_crc(&mut mock, 0x58, &mut buf).unwrap();
        assert_eq!(buf, [0xbe, 0xef, 0x92]);

        // Invalid CRC
        let expectations = [Transaction::read(0x58, vec![0xBE, 0xEF, 0x00])];
        let mut mock = I2cMock::new(&expectations);
        match i2c::read_words_with_crc(&mut mock, 0x58, &mut buf) {
            Err(i2c::Error::Crc) => {}
            Err(_) => panic!("Invalid error: Must be Crc"),
            Ok(_) => panic!("CRC check did not fail"),
        }
        assert_eq!(buf, [0xbe, 0xef, 0x00]); // Buf was changed
    }

    #[test]
    fn write_command() {
        let expectations = [Transaction::write(0x58, vec![0xab, 0xcd])];
        let mut mock = I2cMock::new(&expectations);

        i2c::write_command(&mut mock, 0x58, 0xabcd).unwrap();

        mock.done();
    }
}
