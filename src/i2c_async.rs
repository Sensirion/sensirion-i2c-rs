//! [`embedded-hal-async`] helper functions for I²C communication.
//!
//! These functions are identical to the helpers in the [`i2c`](crate::i2c)
//! module, except that they use the async I²C traits from [`embedded-hal-async`]
//! rther than the blocking I²C traits from `embedded-hal`.
//!
//! [`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

use crate::crc8;
use embedded_hal_async::i2c;

pub use crate::i2c::Error;

/// Write an u8 command to the I²C bus.
pub async fn write_command_u8<I: i2c::I2c>(
    i2c: &mut I,
    addr: u8,
    command: u8,
) -> Result<(), I::Error> {
    i2c.write(addr, &command.to_be_bytes()).await
}

/// Write an u16 command to the I²C bus.
pub async fn write_command_u16<I: i2c::I2c>(
    i2c: &mut I,
    addr: u8,
    command: u16,
) -> Result<(), I::Error> {
    i2c.write(addr, &command.to_be_bytes()).await
}

/// Read data into the provided buffer and validate the CRC8 checksum.
///
/// If the checksum is wrong, return `Error::Crc`.
///
/// # Panics
///
/// This method will consider every third byte a checksum byte. If the buffer size is not a
/// multiple of 3, then it will panic.
pub async fn read_words_with_crc<I: i2c::I2c>(
    i2c: &mut I,
    addr: u8,
    data: &mut [u8],
) -> Result<(), Error<I>> {
    assert!(
        data.len() % 3 == 0,
        "Buffer must hold a multiple of 3 bytes"
    );
    i2c.read(addr, data).await.map_err(Error::I2cRead)?;
    crc8::validate(data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO: `embedded-hal-mock`'s support for `embedded-hal-async` does not
    // currently have a mock I2C implementation. When that's available, we
    // should add tests for the async I2C functions here that are analogous to
    // the ones in the `i2c` module.
}
