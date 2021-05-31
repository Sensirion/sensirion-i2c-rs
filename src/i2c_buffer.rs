//! Buffer for holding data to be sent on the I2C bus. Every third byte contains
//! a crc8 checksum. Invariant: a buffer always contains valid data up to `self.used`.

use crate::{crc8, i2c::Error};
use core::convert::TryInto;
use core::ops::Deref;

/// Append error
#[derive(Debug, PartialEq)]
pub enum AppendError {
    /// Buffer too small for requested operation
    BufferTooSmall,
    /// Invalid input slice size (not a multiple of 2 bytes)
    /// To construct an I2cBuffer from a byte slice, the byte slice must have a multiple of 2 length.
    InvalidBufferSize,
}

impl<W, R> From<AppendError> for Error<W, R>
where
    W: embedded_hal::blocking::i2c::Write,
    R: embedded_hal::blocking::i2c::Read,
{
    fn from(e: AppendError) -> Self {
        match e {
            AppendError::BufferTooSmall => Error::BufferTooSmall,
            AppendError::InvalidBufferSize => Error::InvalidBufferSize,
        }
    }
}

/// A type implementing this trait supports fallibly appending a T to it
pub trait Appendable<T> {
    /// Append a T to a type. If there is insufficient space, do not modify any data
    /// and return with an error.
    fn append(&mut self, val: T) -> Result<(), AppendError>;
}

/// This type wraps a buffer containing i2c data and its checksums.
///
/// ## Usage
///
/// ```
/// use sensirion_i2c::i2c_buffer::I2cBuffer;
/// use sensirion_i2c::i2c_buffer::Appendable;
///
/// let mut data: u16 = 0xC001;
///
/// let i2c_buffer = I2cBuffer::<12>::new().append(data);
/// ```
///
/// ## Panics
/// This buffer panics when constructed with an invalid size.
///
/// ```should_panic
/// use sensirion_i2c::i2c_buffer::I2cBuffer;
///
/// let i2c_buffer = I2cBuffer::<11>::new(); // 11 is not a multiple of 3
/// ```
#[derive(Debug)]
pub struct I2cBuffer<const N: usize> {
    data: [u8; N],
    used: usize,
}

impl<const N: usize> Default for I2cBuffer<N> {
    fn default() -> Self {
        if N % 3 != 0 {
            panic!("Buffer length not a multiple of 3.")
        }
        // Buffer is valid, because `used` is zero.
        Self {
            data: [0u8; N],
            used: 0,
        }
    }
}

impl<const N: usize> I2cBuffer<N> {
    /// Construct an empty I2cBuffer
    pub fn new() -> Self {
        Default::default()
    }

    /// Validate the buffer content using crc8
    pub fn validate(&self) -> Result<(), crc8::CrcError> {
        crc8::validate(&self.data[..self.used])
    }

    /// Write this I2cBuffer to the given address on the given I2C bus
    pub fn write<I2cWrite: embedded_hal::blocking::i2c::Write>(
        &self,
        addr: u8,
        i2c: &mut I2cWrite,
    ) -> Result<(), I2cWrite::Error> {
        i2c.write(addr, &self.data[..self.used])
    }

    /// Read into this I2cBuffer from the given address on the given I2C bus.
    /// Validate the data using crc8 on every 16-bit word.
    pub fn read_and_validate<
        I2cWrite: embedded_hal::blocking::i2c::Write,
        I2cRead: embedded_hal::blocking::i2c::Read,
    >(
        &mut self,
        addr: u8,
        i2c: &mut I2cRead,
    ) -> Result<(), Error<I2cWrite, I2cRead>> {
        if let Err(e) = i2c.read(addr, &mut self.data[..N]) {
            self.used = 0;
            return Err(Error::I2cRead(e));
        }
        self.used = N;
        if self.validate().is_err() {
            self.used = 0;
            return Err(Error::CrcError);
        }
        Ok(())
    }

    /// Get a reference to the element at `index`, or None if trying to get an
    /// unused element.
    pub fn get(&self, index: usize) -> Option<&u8> {
        if index < self.used {
            Some(&self.data[index])
        } else {
            None
        }
    }
}

impl<const N: usize> Deref for I2cBuffer<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.used]
    }
}

impl<const N: usize> Appendable<[u8; 2]> for I2cBuffer<N> {
    fn append(&mut self, val: [u8; 2]) -> Result<(), AppendError> {
        let crc8 = crc8::calculate(&val);
        if self.used <= self.data.len() - 3 {
            self.data[self.used] = val[0];
            self.data[self.used + 1] = val[1];
            self.data[self.used + 2] = crc8;
            self.used += 3;
            Ok(())
        } else {
            Err(AppendError::BufferTooSmall)
        }
    }
}

impl<const N: usize> Appendable<u16> for I2cBuffer<N> {
    fn append(&mut self, val: u16) -> Result<(), AppendError> {
        self.append(val.to_be_bytes())
    }
}

impl<const N: usize> Appendable<u32> for I2cBuffer<N> {
    fn append(&mut self, val: u32) -> Result<(), AppendError> {
        if self.used + 6 > self.data.len() {
            return Err(AppendError::BufferTooSmall);
        }
        let arr: [u8; 4] = val.to_be_bytes();
        let (arr1, arr2) = arr.split_at(2);
        let arr1: [u8; 2] = arr1.try_into().unwrap();
        let arr2: [u8; 2] = arr2.try_into().unwrap();
        self.append(arr1)?;
        self.append(arr2)?;
        Ok(())
    }
}

impl<const N: usize> Appendable<f32> for I2cBuffer<N> {
    fn append(&mut self, val: f32) -> Result<(), AppendError> {
        if self.used + 6 > self.data.len() {
            return Err(AppendError::BufferTooSmall);
        }
        let arr: [u8; 4] = val.to_be_bytes();
        let (arr1, arr2) = arr.split_at(2);
        let arr1: [u8; 2] = arr1.try_into().unwrap();
        let arr2: [u8; 2] = arr2.try_into().unwrap();
        self.append(arr1)?;
        self.append(arr2)?;
        Ok(())
    }
}

impl<const N: usize> Appendable<&[u16]> for I2cBuffer<N> {
    fn append(&mut self, val: &[u16]) -> Result<(), AppendError> {
        let num_elems = val.len() * 3;
        if self.used + num_elems > self.data.len() {
            return Err(AppendError::BufferTooSmall);
        }
        for v in val.iter() {
            match self.append(*v) {
                Ok(_) => {}
                Err(_) => unreachable!(),
            }
        }
        Ok(())
    }
}

impl<const N: usize> Appendable<&[u8]> for I2cBuffer<N> {
    fn append(&mut self, val: &[u8]) -> Result<(), AppendError> {
        if val.len() % 2 != 0 {
            return Err(AppendError::InvalidBufferSize);
        }
        let num_elems = val.len() + val.len() / 2;
        if self.used + num_elems > self.data.len() {
            return Err(AppendError::BufferTooSmall);
        }
        for v in val.chunks(2) {
            // TODO do this by calling Appendable::<[u8; 2]>::append(..)
            let crc8 = crc8::calculate(&v);
            self.data[self.used] = v[0];
            self.data[self.used + 1] = v[1];
            self.data[self.used + 2] = crc8;
            self.used += 3;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use crate::{crc8, i2c::Error, i2c_buffer};

    use embedded_hal_mock as hal;
    use hal::{
        i2c::{Mock as I2cMock, Transaction},
        MockError,
    };
    use i2c_buffer::{AppendError, Appendable, I2cBuffer};

    #[test]
    #[should_panic(expected = "Buffer length not a multiple of 3.")]
    fn buffer_invalid_fails() {
        let _ = I2cBuffer::<32>::new();
    }

    #[test]
    fn buffer_default_valid() {
        let buffer = I2cBuffer::<12>::new();
        buffer.validate().unwrap();
    }

    #[test]
    fn buffer_zero_valid() {
        let mut buffer = I2cBuffer::<6>::new();
        buffer.append(0u16).unwrap();
        buffer.append(0u16).unwrap();
        buffer.validate().unwrap();
    }

    #[test]
    fn buffer_validate_u16() {
        let data: u16 = 0xbeef;
        let mut buffer = I2cBuffer::<3>::new();
        buffer.append(data).unwrap();
        buffer.validate().unwrap();
    }

    #[test]
    fn make_i2c_buffer() {
        let mut i2c_buffer = I2cBuffer::<9>::new();
        i2c_buffer.append(0x0102u16).unwrap();
    }

    #[test]
    fn buffer_with_u32() {
        let mut i2c_buffer = I2cBuffer::<6>::new();
        i2c_buffer.append(0x1C0FFEE1u32).unwrap();
        assert!(i2c_buffer.validate().is_ok());
    }

    #[test]
    fn buffer_with_f32() {
        let mut i2c_buffer = I2cBuffer::<6>::new();
        i2c_buffer.append(12345.678f32).unwrap();
        assert!(i2c_buffer.validate().is_ok());
    }

    #[test]
    fn write_i2c_buffer() {
        let crc = crc8::calculate(&[0xab, 0xcd]);
        let expectations = [Transaction::write(0x58, vec![0xab, 0xcd, crc])];
        let mut mock = I2cMock::new(&expectations);

        let mut i2c_buffer = I2cBuffer::<30>::new();
        i2c_buffer.append([0xab, 0xcd]).unwrap();

        i2c_buffer.write(0x58, &mut mock).unwrap();
        mock.done();
    }

    #[test]
    fn read_i2c_buffer_ok() {
        let crc = crc8::calculate(&[0xab, 0xcd]);
        let expectations = [Transaction::read(
            0x12,
            vec![0xab, 0xcd, crc, 0xab, 0xcd, crc],
        )];
        let mut mock = I2cMock::new(&expectations);

        let mut i2c_buffer = I2cBuffer::<6>::new();

        i2c_buffer
            .read_and_validate::<I2cMock, I2cMock>(0x12, &mut mock)
            .unwrap();
        assert!(i2c_buffer.validate().is_ok());
        mock.done();
    }

    #[test]
    fn read_i2c_buffer_err() {
        let crc = crc8::calculate(&[0xab, 0xcd]);
        let expectations = [
            Transaction::read(0x12, vec![0xab, 0xcd, crc, 0xab, 0xcd, crc])
                .with_error(MockError::Io(ErrorKind::Other)),
        ];
        let mut mock = I2cMock::new(&expectations);

        let mut i2c_buffer = I2cBuffer::<6>::new();

        match i2c_buffer.read_and_validate::<I2cMock, I2cMock>(0x12, &mut mock) {
            Err(Error::I2cRead(MockError::Io(ErrorKind::Other))) => {}
            Ok(_) => panic!("Succeeded but should have failed"),
            _ => panic!("Invalid error variant"),
        }
        assert!(i2c_buffer.validate().is_ok());
        mock.done();
    }

    #[test]
    fn read_i2c_buffer_error() {
        let crc = crc8::calculate(&[0xab, 0xcd]);
        let expectations = [Transaction::read(
            0x12,
            vec![0xab, 0xcd, 0x01, 0xab, 0xcd, crc],
        )];
        let mut mock = I2cMock::new(&expectations);

        let mut i2c_buffer = I2cBuffer::<6>::new();

        match i2c_buffer.read_and_validate::<I2cMock, I2cMock>(0x12, &mut mock) {
            Err(Error::CrcError) => {}
            Ok(_) => panic!("Crc check did not fail"),
            _ => panic!("Wrong error variant"),
        }
        assert_eq!(i2c_buffer.validate(), Ok(()));
        mock.done();
    }

    #[test]
    fn buffer_too_short() {
        let mut i2c_buffer = I2cBuffer::new();
        let data: u32 = 0xc0bfefe;
        for i in 0..6 {
            i2c_buffer.append(data + i).unwrap();
        }
        assert_eq!(
            i2c_buffer.append(0xfefeu16),
            Err(AppendError::BufferTooSmall)
        );
        assert_eq!(i2c_buffer.append(0.5f32), Err(AppendError::BufferTooSmall));
        assert!(i2c_buffer.validate().is_ok());
        let expected: [u8; 36] = [
            0x0c, 0x0b, 0xdf, 0xfe, 0xfe, 0x69, 0x0c, 0x0b, 0xdf, 0xfe, 0xff, 0x58, 0x0c, 0x0b,
            0xdf, 0xff, 0x00, 0x00, 0x0c, 0x0b, 0xdf, 0xff, 0x01, 0x31, 0x0c, 0x0b, 0xdf, 0xff,
            0x02, 0x62, 0x0c, 0x0b, 0xdf, 0xff, 0x03, 0x53,
        ];
        assert_eq!(expected, i2c_buffer.data);
        assert_eq!(36, i2c_buffer.used)
    }

    #[test]
    fn buffer_append_f32() {
        let mut i2c_buffer = I2cBuffer::new();
        let data: u32 = 0x0c0bfefe;
        for i in 0..6 {
            i2c_buffer.append(data + i).unwrap();
        }
        assert_eq!(
            i2c_buffer.append(&[0xc0deu16, 0xba1d][..]),
            Err(AppendError::BufferTooSmall)
        );
        assert!(i2c_buffer.validate().is_ok());

        let expected: [u8; 36] = [
            0x0c, 0x0b, 0xdf, 0xfe, 0xfe, 0x69, 0x0c, 0x0b, 0xdf, 0xfe, 0xff, 0x58, 0x0c, 0x0b,
            0xdf, 0xff, 0x00, 0x00, 0x0c, 0x0b, 0xdf, 0xff, 0x01, 0x31, 0x0c, 0x0b, 0xdf, 0xff,
            0x02, 0x62, 0x0c, 0x0b, 0xdf, 0xff, 0x03, 0x53,
        ];
        assert_eq!(expected, i2c_buffer.data);
        assert_eq!(36, i2c_buffer.used);
    }

    #[test]
    fn buffer_append_u32() {
        let mut i2c_buffer = I2cBuffer::new();
        let data: u32 = 0x0c0bfefe;
        for i in 0..6 {
            i2c_buffer.append(data + i).unwrap();
        }
        assert_eq!(
            i2c_buffer.append(&[0xc0deu16, 0xba1d][..]),
            Err(AppendError::BufferTooSmall)
        );
        assert!(i2c_buffer.validate().is_ok());

        // Assert the buffer is unchanged by the operation that did not succeed
        let expected: [u8; 36] = [
            0x0c, 0x0b, 0xdf, 0xfe, 0xfe, 0x69, 0x0c, 0x0b, 0xdf, 0xfe, 0xff, 0x58, 0x0c, 0x0b,
            0xdf, 0xff, 0x00, 0x00, 0x0c, 0x0b, 0xdf, 0xff, 0x01, 0x31, 0x0c, 0x0b, 0xdf, 0xff,
            0x02, 0x62, 0x0c, 0x0b, 0xdf, 0xff, 0x03, 0x53,
        ];
        assert_eq!(expected, i2c_buffer.data);
        assert_eq!(36, i2c_buffer.used);
    }

    #[test]
    fn make_buffer_from_u16_slice() {
        let mut i2c_buffer = I2cBuffer::new();
        let data: [u16; 10] = [
            0xcafe, 0xbabe, 0xdead, 0xbeef, 0xc0de, 0xacab, 0xf00d, 0xb00b, 0xd00d, 0xface,
        ];
        i2c_buffer.append(&data[..]).unwrap();
        assert_eq!(
            i2c_buffer.append(0xfefeu16),
            Err(AppendError::BufferTooSmall)
        );
        assert_eq!(
            i2c_buffer.append(0xc0c0fefeu32),
            Err(AppendError::BufferTooSmall)
        );
        assert!(i2c_buffer.validate().is_ok());

        // Assert the buffer is unchanged by the operation that did not succeed
        let expected: [u8; 30] = [
            0xca, 0xfe, 0x58, 0xba, 0xbe, 0x5e, 0xde, 0xad, 0x98, 0xbe, 0xef, 0x92, 0xc0, 0xde,
            0x30, 0xac, 0xab, 0xdc, 0xf0, 0x0d, 0xd5, 0xb0, 0x0b, 0xfa, 0xd0, 0x0d, 0x09, 0xfa,
            0xce, 0x2f,
        ];
        assert_eq!(expected, i2c_buffer.data);
        assert_eq!(30, i2c_buffer.used);
    }

    #[test]
    fn make_buffer_from_u8_slice() {
        let mut i2c_buffer = I2cBuffer::<30>::new();
        let data: [u8; 20] = [
            0xca, 0xfe, 0xba, 0xbe, 0xde, 0xad, 0xbe, 0xef, 0xc0, 0xde, 0xac, 0xab, 0xf0, 0x0d,
            0xb0, 0x0b, 0xd0, 0x0d, 0xfa, 0xce,
        ];
        i2c_buffer.append(&data[..]).unwrap();
        let copy = i2c_buffer.data.clone();
        assert_eq!(
            i2c_buffer.append(0xfefeu16),
            Err(AppendError::BufferTooSmall)
        );
        assert_eq!(
            i2c_buffer.append(&[0xabu8, 0xcd][..]),
            Err(AppendError::BufferTooSmall)
        );
        assert!(i2c_buffer.validate().is_ok());

        // Assert the buffer is unchanged by the operation that did not succeed
        assert_eq!(copy, i2c_buffer.data);
        assert_eq!(30, i2c_buffer.used);
    }

    #[test]
    fn invalid_u8_slice_length() {
        let mut i2c_buffer = I2cBuffer::<12>::new();
        let data: [u8; 14] = [
            0xca, 0xfe, 0xba, 0xbe, 0xde, 0xad, 0xbe, 0xef, 0xc0, 0xde, 0xb0, 0x0b, 0xba, 0x1d,
        ];
        let copy = i2c_buffer.data.clone();
        assert_eq!(
            Err(AppendError::BufferTooSmall),
            i2c_buffer.append(&data[..])
        );

        // Assert the buffer is unchanged by the operation that did not succeed
        assert_eq!(copy, i2c_buffer.data);
        assert_eq!(0, i2c_buffer.used);
    }

    #[test]
    fn invalid_u8_slice() {
        let mut i2c_buffer = I2cBuffer::<24>::new();
        let data: [u8; 13] = [
            0xca, 0xfe, 0xba, 0xbe, 0xde, 0xad, 0xbe, 0xef, 0xc0, 0xde, 0xac, 0xab, 0xf0,
        ];
        assert_eq!(
            Err(AppendError::InvalidBufferSize),
            i2c_buffer.append(&data[..])
        );

        // Assert the buffer is unchanged by the operation that did not succeed
        let expected = [0u8; 24];
        assert_eq!(expected, i2c_buffer.data);
        assert_eq!(0, i2c_buffer.used);
    }

    #[test]
    fn test_get_element() {
        let mut i2c_buffer = I2cBuffer::<6>::new();
        i2c_buffer.append(0xefdeu16).unwrap();
        assert_eq!(i2c_buffer.get(0).unwrap(), &0xef);
        assert_eq!(i2c_buffer.get(1).unwrap(), &0xde);
        assert!(i2c_buffer.get(3).is_none());
        assert!(i2c_buffer.get(4).is_none());
        assert!(i2c_buffer.get(5).is_none());
    }

    #[test]
    fn test_deref() {
        let buf: I2cBuffer<6> = I2cBuffer::new();
        assert_eq!(0, buf.len());
        let mut buf: I2cBuffer<9> = I2cBuffer::new();
        buf.append(15u16).unwrap();
        assert_eq!(3, buf.len());
        assert_eq!([0u8, 15, 175], *buf);
        buf.append(18u16).unwrap();
        assert_eq!(6, buf.len());
        assert_eq!([0u8, 15, 175, 0, 18, 160], *buf);
    }
}
