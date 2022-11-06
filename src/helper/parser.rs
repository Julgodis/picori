use std::borrow::Borrow;
use std::panic::Location;

use super::endian::{BigEndian, EndianAgnostic, LittleEndian, NativeEndian};
use super::Reader;
use crate::Result;

/// A helper trait for types that can interpret bytes.
pub trait Parser: Reader {
    /// Read a single u8.
    #[track_caller]
    #[inline]
    fn deserialize_u8(&mut self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.read_into_tracked(&mut buffer, Location::caller())?;
        Ok(buffer[0])
    }

    /// Read a single endian agnostic u16.
    fn deserialize_eu16<E: EndianAgnostic>(&mut self, caller: &'static Location) -> Result<u16> {
        let mut buffer = [0u8; 2];
        self.read_into_tracked(&mut buffer, caller)?;
        Ok(E::u16_from_bytes(&buffer))
    }

    /// Read a single endian agnostic u32.
    fn deserialize_eu32<E: EndianAgnostic>(&mut self, caller: &'static Location) -> Result<u32> {
        let mut buffer = [0u8; 4];
        self.read_into_tracked(&mut buffer, caller)?;
        Ok(E::u32_from_bytes(&buffer))
    }

    /// Read a single u16 in native endian.
    #[track_caller]
    #[inline]
    fn deserialize_u16(&mut self) -> Result<u16> {
        self.deserialize_eu16::<NativeEndian>(Location::caller())
    }

    /// Read a single u32 in native endian.
    #[track_caller]
    #[inline]
    fn deserialize_u32(&mut self) -> Result<u32> {
        self.deserialize_eu32::<NativeEndian>(Location::caller())
    }

    /// Read a single u16 in big endian.
    #[track_caller]
    #[inline]
    fn deserialize_bu16(&mut self) -> Result<u16> {
        self.deserialize_eu16::<BigEndian>(Location::caller())
    }

    /// Read a single u32 in big endian.
    #[track_caller]
    #[inline]
    fn deserialize_bu32(&mut self) -> Result<u32> {
        self.deserialize_eu32::<BigEndian>(Location::caller())
    }

    /// Read a single u16 in little endian.
    #[track_caller]
    #[inline]
    fn deserialize_lu16(&mut self) -> Result<u16> {
        self.deserialize_eu16::<LittleEndian>(Location::caller())
    }

    /// Read a single u32 in little endian.
    #[track_caller]
    #[inline]
    fn deserialize_lu32(&mut self) -> Result<u32> {
        self.deserialize_eu32::<LittleEndian>(Location::caller())
    }

    /// Read string with the given encoding until the NUL character is
    /// encountered or `L` bytes have been read.
    #[track_caller]
    fn deserialize_str<const L: usize, E: ParseStringEncoding>(&mut self) -> Result<String> {
        let mut buffer = [0u8; L];
        self.read_into_tracked(&mut buffer, Location::caller())?;
        E::parse_str(buffer)
    }

    /// Read array of u8 with the given length `L`.
    #[track_caller]
    #[inline]
    fn deserialize_u8_array<const L: usize>(&mut self) -> Result<[u8; L]> {
        self.read_buffer_of_tracked::<u8, L>(Location::caller())
    }

    /// Read array of endian agnostic u16 with the given length `L`.
    #[inline]
    fn deserialize_eu16_array<E: EndianAgnostic, const L: usize>(
        &mut self,
        caller: &'static Location,
    ) -> Result<[u16; L]> {
        let mut buf = self.read_buffer_of_tracked::<u16, L>(caller)?;
        for value in buf.iter_mut().take(L) {
            *value = E::u16_from_native(*value);
        }
        Ok(buf)
    }

    /// Read array of big endian u16 with the given length `L`.
    #[track_caller]
    #[inline]
    fn deserialize_bu16_array<const L: usize>(&mut self) -> Result<[u16; L]> {
        self.deserialize_eu16_array::<BigEndian, L>(Location::caller())
    }

    /// Read array of little endian u16 with the given length `L`.
    #[track_caller]
    #[inline]
    fn deserialize_lu16_array<const L: usize>(&mut self) -> Result<[u16; L]> {
        self.deserialize_eu16_array::<LittleEndian, L>(Location::caller())
    }

    /// Read array of endian agnostic u32 with the given length `L`.
    #[inline]
    fn deserialize_eu32_array<E: EndianAgnostic, const L: usize>(
        &mut self,
        caller: &'static Location,
    ) -> Result<[u32; L]> {
        let mut buf = self.read_buffer_of_tracked::<u32, L>(caller)?;
        for value in buf.iter_mut().take(L) {
            *value = E::u32_from_native(*value);
        }
        Ok(buf)
    }

    /// Read array of big endian u32 with the given length `L`.
    #[track_caller]
    #[inline]
    fn deserialize_bu32_array<const L: usize>(&mut self) -> Result<[u32; L]> {
        self.deserialize_eu32_array::<BigEndian, L>(Location::caller())
    }

    /// Read array of little endian u32 with the given length `L`.
    #[track_caller]
    #[inline]
    fn deserialize_lu32_array<const L: usize>(&mut self) -> Result<[u32; L]> {
        self.deserialize_eu32_array::<LittleEndian, L>(Location::caller())
    }
}

/// Implementation of [`Parser`] for all [`Reader`].
impl<Base: Reader> Parser for Base {}

pub trait ParseStringEncoding {
    fn parse_str<I>(data: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized;
}

// -------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8() {
        let mut data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        assert_eq!(data.deserialize_u8().unwrap(), 0x01);
        assert_eq!(data.deserialize_u8().unwrap(), 0x02);
        assert_eq!(data.deserialize_u8().unwrap(), 0x03);
        assert_eq!(data.deserialize_u8().unwrap(), 0x04);
    }

    #[test]
    fn u16() {
        let mut data: &[u8] = &[0x01, 0x02, 0x01, 0x02];
        assert_eq!(data.deserialize_bu16().unwrap(), 0x0102);
        assert_eq!(data.deserialize_lu16().unwrap(), 0x0201);
    }

    #[test]
    fn u32() {
        let mut data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04];
        assert_eq!(data.deserialize_bu32().unwrap(), 0x01020304);
        assert_eq!(data.deserialize_lu32().unwrap(), 0x04030201);
    }

    #[test]
    fn u8_array() {
        let mut data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        assert_eq!(data.deserialize_u8_array::<4>().unwrap(), [
            0x01, 0x02, 0x03, 0x04
        ]);
    }

    #[test]
    fn u16_array() {
        let mut data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04];
        assert_eq!(data.deserialize_bu16_array::<2>().unwrap(), [
            0x0102, 0x0304,
        ]);
        assert_eq!(data.deserialize_lu16_array::<2>().unwrap(), [
            0x0201, 0x0403,
        ]);
    }

    #[test]
    fn u32_array() {
        let mut data: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
            0x07, 0x08,
        ];
        assert_eq!(data.deserialize_bu32_array::<2>().unwrap(), [
            0x01020304, 0x05060708
        ]);
        assert_eq!(data.deserialize_lu32_array::<2>().unwrap(), [
            0x04030201, 0x08070605
        ]);
    }
}
