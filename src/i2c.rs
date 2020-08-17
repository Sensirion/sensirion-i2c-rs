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
    i2c.try_write(addr, &command.to_be_bytes())
}

/// Read data into the provided buffer and validate the CRC8 checksum.
///
/// If the checksum is wrong, return `Error::Crc`.
///
/// Note: This method will consider every third byte a checksum byte. If
/// the buffer size is not a multiple of 3, then not all data will be
/// validated.
pub fn read_words_with_crc<I2c: i2c::Read + i2c::Write>(
    i2c: &mut I2c,
    addr: u8,
    data: &mut [u8],
) -> Result<(), Error<I2c, I2c>> {
    debug_assert!(
        data.len() % 3 == 0,
        "Buffer must hold a multiple of 3 bytes"
    );
    i2c.try_read(addr, data).map_err(Error::I2cRead)?;
    crc8::validate(data).map_err(|_| Error::Crc)
}
