# Changelog

This project follows [semantic versioning](https://semver.org/).

## [Unreleased]

### Changed

 * Panic in `crc8::validate` and `i2c::read_words_with_crc` if buffer size is
   not a multiple of 3
   ([#13](https://github.com/Sensirion/sensirion-i2c-rs/pull/13),
   [#15](https://github.com/Sensirion/sensirion-i2c-rs/pull/15))

## [0.1.0] (2020-08-21)

 * Initial version which implements the CRC-8 algorithm commonly used by
   Sensirion's sensors and a few I2C helper functions.

[0.1.0]: https://github.com/Sensirion/sensirion-i2c-rs/releases/tag/v0.1.0
