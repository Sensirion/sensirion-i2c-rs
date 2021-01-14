# Changelog

This project follows [semantic versioning](https://semver.org/).

## [Unreleased]

### Changed

 * Add a separate error type for the crc8 module to allow for
   `From<crc8::Error>` implementations.
   ([#18](https://github.com/Sensirion/sensirion-i2c-rs/pull/18/))

## [0.1.1] (2020-09-02)

### Changed

 * Panic in `crc8::validate` and `i2c::read_words_with_crc` if buffer size is
   not a multiple of 3
   ([#13](https://github.com/Sensirion/sensirion-i2c-rs/pull/13),
   [#15](https://github.com/Sensirion/sensirion-i2c-rs/pull/15))

## [0.1.0] (2020-08-21)

 * Initial version which implements the CRC-8 algorithm commonly used by
   Sensirion's sensors and a few I2C helper functions.

[Unreleased]: https://github.com/Sensirion/sensirion-i2c-rs/compare/v0.1.1..master
[0.1.1]: https://github.com/Sensirion/sensirion-i2c-rs/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/Sensirion/sensirion-i2c-rs/releases/tag/v0.1.0
