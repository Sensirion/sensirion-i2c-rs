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
//! use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
//! use sensirion_i2c::i2c;
//!
//! let expectations = [I2cTransaction::write(0x12, vec![0x34, 0x56])];
//! let mut i2c_mock = I2cMock::new(&expectations);
//! i2c::write_command_u16(&mut i2c_mock, 0x12, 0x3456);
//! i2c_mock.done();
//! ```
//!
//! #### `embedded-hal-async`
//!
//! The `i2c_async` module provides versions of the I2C helpers in this crate
//! that use the [`embedded-hal-async`
//! crate](https://crates.io/crates/embedded-hal-async), rather than the
//! blocking I2C traits from `embedded-hal`.
//!
//! This module is only available when the `embedded-hal-async`
//! Cargo feature is enabled.

#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod crc8;
pub mod i2c;
#[cfg(feature = "embedded-hal-async")]
pub mod i2c_async;
