//! Library with common functionality for I2C based sensors from Sensirion, based on the
//! `embedded-hal` traits.
//!
//! ## Usage
//!
//! ### CRC8 calculation / validation
//!
//! ```
//! use sensirion_i2c::crc8;
//!
//! let data = [0xbe, 0xef];
//! let crc = crc8::calculate(&data);
//!
//! let data = [0xbe, 0xef, crc];
//! assert_eq!(Ok(()), crc8::validate(&data));
//! ```
//!
//! ### I2C
//!
//! The I2C helpers work with any `embedded_hal::i2c` implementation.
//!
//! ```
//! use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//! use sensirion_i2c::i2c;
//!
//! let expectations = [I2cTransaction::write(0x12, vec![0x34, 0x56])];
//! let mut i2c_mock = I2cMock::new(&expectations);
//! i2c::write_command(&mut i2c_mock, 0x12, 0x3456);
//! ```

#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;

/// All possible errors in this crate
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error<I2cWrite: Write, I2cRead: Read> {
    I2cWrite(I2cWrite::Error),
    I2cRead(I2cRead::Error),
    WrongBufferSize,
    WrongCrc,
}

impl<I2cWrite: Write, I2cRead: Read> From<crc8::Error> for Error<I2cWrite, I2cRead> {
    fn from(err: crc8::Error) -> Self {
        match err {
            crc8::Error::WrongBufferSize => Error::WrongBufferSize,
            crc8::Error::WrongCrc => Error::WrongCrc,
        }
    }
}

pub mod crc8;
pub mod i2c;
