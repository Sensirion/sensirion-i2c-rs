#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

pub mod crc;
pub mod i2c;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
