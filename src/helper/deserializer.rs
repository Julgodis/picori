use super::endian::{BigEndian, EndianAgnostic, LittleEndian, NativeEndian};
use super::{DeserializableStringEncoding, Reader};
use crate::PicoriError;

pub trait Deserializer: Reader {
    fn deserialize_u8(&mut self) -> Result<u8, PicoriError>;
    fn deserialize_eu16<E: EndianAgnostic>(&mut self) -> Result<u16, PicoriError>;
    fn deserialize_eu32<E: EndianAgnostic>(&mut self) -> Result<u32, PicoriError>;

    #[inline]
    fn deserialize_u16(&mut self) -> Result<u16, PicoriError> {
        self.deserialize_eu16::<NativeEndian>()
    }

    #[inline]
    fn deserialize_u32(&mut self) -> Result<u32, PicoriError> {
        self.deserialize_eu32::<NativeEndian>()
    }

    #[inline]
    fn deserialize_bu16(&mut self) -> Result<u16, PicoriError> {
        self.deserialize_eu16::<BigEndian>()
    }

    #[inline]
    fn deserialize_bu32(&mut self) -> Result<u32, PicoriError> {
        self.deserialize_eu32::<BigEndian>()
    }

    #[inline]
    fn deserialize_lu16(&mut self) -> Result<u16, PicoriError> {
        self.deserialize_eu16::<LittleEndian>()
    }

    #[inline]
    fn deserialize_lu32(&mut self) -> Result<u32, PicoriError> {
        self.deserialize_eu32::<LittleEndian>()
    }

    fn deserialize_str<const L: usize, E: DeserializableStringEncoding>(
        &mut self,
    ) -> Result<String, PicoriError> {
        let buf = self.read_fixed_buffer::<L>()?;
        let str = E::deserialize_str(buf)?;
        Ok(str)
    }

    #[inline]
    fn deserialize_u8_array<const L: usize>(&mut self) -> Result<[u8; L], PicoriError> {
        Ok(self.read_fixed_buffer::<L>()?)
    }

    fn deserialize_eu16_array<E: EndianAgnostic, const L: usize>(
        &mut self,
    ) -> Result<[u16; L], PicoriError> {
        let mut buf = self.read_fixed_buffer_cge::<u16, L>()?;
        for i in 0..L {
            buf[i] = E::u16_from_native(buf[i]);
        }
        Ok(buf)
    }

    #[inline]
    fn deserialize_bu16_array<const L: usize>(&mut self) -> Result<[u16; L], PicoriError> {
        self.deserialize_eu16_array::<BigEndian, L>()
    }

    #[inline]
    fn deserialize_lu16_array<const L: usize>(&mut self) -> Result<[u16; L], PicoriError> {
        self.deserialize_eu16_array::<LittleEndian, L>()
    }

    fn deserialize_eu32_array<E: EndianAgnostic, const L: usize>(
        &mut self,
    ) -> Result<[u32; L], PicoriError> {
        let mut buf = self.read_fixed_buffer_cge::<u32, L>()?;
        for i in 0..L {
            buf[i] = E::u32_from_native(buf[i]);
        }
        Ok(buf)
    }

    #[inline]
    fn deserialize_bu32_array<const L: usize>(&mut self) -> Result<[u32; L], PicoriError> {
        self.deserialize_eu32_array::<BigEndian, L>()
    }

    #[inline]
    fn deserialize_lu32_array<const L: usize>(&mut self) -> Result<[u32; L], PicoriError> {
        self.deserialize_eu32_array::<LittleEndian, L>()
    }
}

impl<Base> Deserializer for Base
where
    Base: Reader,
{
    #[inline]
    fn deserialize_u8(&mut self) -> Result<u8, PicoriError> {
        let buf = self.read_fixed_buffer::<1>()?;
        Ok(buf[0])
    }

    fn deserialize_eu16<E: EndianAgnostic>(&mut self) -> Result<u16, PicoriError> {
        let buf = self.read_fixed_buffer::<2>()?;
        Ok(E::u16_from_bytes(&buf))
    }

    fn deserialize_eu32<E: EndianAgnostic>(&mut self) -> Result<u32, PicoriError> {
        let buf = self.read_fixed_buffer::<4>()?;
        Ok(E::u32_from_bytes(&buf))
    }
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
