pub enum LittleEndian {}

pub enum BigEndian {}

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

pub trait EndianAgnostic {
    fn u16_from_bytes(bytes: &[u8]) -> u16;
    fn u32_from_bytes(bytes: &[u8]) -> u32;

    fn u16_from_native(n: u16) -> u16;
    fn u32_from_native(n: u32) -> u32;
}

impl EndianAgnostic for LittleEndian {
    #[inline]
    fn u16_from_bytes(bytes: &[u8]) -> u16 {
        debug_assert!(bytes.len() == 2);
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline]
    fn u32_from_bytes(bytes: &[u8]) -> u32 {
        debug_assert!(bytes.len() == 4);
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    #[inline]
    fn u16_from_native(n: u16) -> u16 { n.to_le() }

    #[inline]
    fn u32_from_native(n: u32) -> u32 { n.to_le() }
}

impl EndianAgnostic for BigEndian {
    #[inline]
    fn u16_from_bytes(bytes: &[u8]) -> u16 {
        debug_assert!(bytes.len() == 2);
        u16::from_be_bytes(bytes.try_into().unwrap())
    }

    #[inline]
    fn u32_from_bytes(bytes: &[u8]) -> u32 {
        debug_assert!(bytes.len() == 4);
        u32::from_be_bytes(bytes.try_into().unwrap())
    }

    #[inline]
    fn u16_from_native(n: u16) -> u16 { n.to_be() }

    #[inline]
    fn u32_from_native(n: u32) -> u32 { n.to_be() }
}

// -------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_endian() {
        assert_eq!(BigEndian::u16_from_bytes(&[0x01, 0x02]), 0x0102);
        assert_eq!(
            BigEndian::u32_from_bytes(&[0x01, 0x02, 0x03, 0x04]),
            0x01020304
        );
    }

    #[test]
    fn little_endian() {
        assert_eq!(LittleEndian::u16_from_bytes(&[0x01, 0x02]), 0x0201);
        assert_eq!(
            LittleEndian::u32_from_bytes(&[0x01, 0x02, 0x03, 0x04]),
            0x04030201
        );
    }
}
