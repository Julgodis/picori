pub enum LittleEndian {}

pub enum BigEndian {}

#[cfg(target_endian = "little")]
pub type NativeEndian = LittleEndian;

#[cfg(target_endian = "big")]
pub type NativeEndian = BigEndian;

pub trait EndianAgnostic {
    fn as_u8(data: &[u8]) -> u8;
    fn as_u16(data: &[u8]) -> u16;
    fn as_u32(data: &[u8]) -> u32;
    fn as_u64(data: &[u8]) -> u64;
}

impl EndianAgnostic for LittleEndian {
    #[inline]
    fn as_u8(data: &[u8]) -> u8 { data[0] }
    #[inline]
    fn as_u16(data: &[u8]) -> u16 { u16::from_le_bytes(data[..2].try_into().unwrap()) }
    #[inline]
    fn as_u32(data: &[u8]) -> u32 { u32::from_le_bytes(data[..4].try_into().unwrap()) }
    #[inline]
    fn as_u64(data: &[u8]) -> u64 { u64::from_le_bytes(data[..8].try_into().unwrap()) }
}

impl EndianAgnostic for BigEndian {
    #[inline]
    fn as_u8(data: &[u8]) -> u8 { data[0] }
    #[inline]
    fn as_u16(data: &[u8]) -> u16 { u16::from_be_bytes(data[..2].try_into().unwrap()) }
    #[inline]
    fn as_u32(data: &[u8]) -> u32 { u32::from_be_bytes(data[..4].try_into().unwrap()) }
    #[inline]
    fn as_u64(data: &[u8]) -> u64 { u64::from_be_bytes(data[..8].try_into().unwrap()) }
}
