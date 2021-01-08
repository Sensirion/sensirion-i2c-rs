//! Library with common functionality for I2C based sensors from Sensirion, based on the
//! `embedded-hal` traits.
//!
//! ## Usage
//!
//! ### CRC8 calculation / validation
//!
//! ```
//! use sensirion_i2c::crc8;
//! use sensirion_i2c::i2c_buffer::I2cBuffer;
//! use sensirion_i2c::i2c_buffer::Appendable;
//!
//! let data: u16 = 0xbeef;
//!
//! let mut i2c_buffer = I2cBuffer::<12>::new();
//! i2c_buffer.append(data).unwrap();
//!
//! assert!(i2c_buffer.validate().is_ok());
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

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

pub mod crc8;
pub mod i2c;
pub mod i2c_buffer;
