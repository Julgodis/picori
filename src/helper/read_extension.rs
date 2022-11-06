use std::{io::Read, mem::MaybeUninit};

use crate::{error::PicoriError, endian::{LittleEndian, BigEndian, EndianAgnostic, NativeEndian}, string::StringEncoding};


pub trait ReadExtensionU8: Read {
    fn read_eu8<T: EndianAgnostic>(&mut self) -> Result<u8, PicoriError> {
        let mut buf = MaybeUninit::<[u8; 1]>::uninit();
        let slice = unsafe { &mut *buf.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(T::as_u8(unsafe { &buf.assume_init() }))
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, PicoriError> { self.read_eu8::<NativeEndian>() }
    #[inline]
    fn read_bu8(&mut self) -> Result<u8, PicoriError> { self.read_eu8::<BigEndian>() }
    #[inline]
    fn read_lu8(&mut self) -> Result<u8, PicoriError> { self.read_eu8::<LittleEndian>() }
}

pub trait ReadExtensionU32: Read {
    fn read_eu32<T: EndianAgnostic>(&mut self) -> Result<u32, PicoriError> {
        let mut buf = MaybeUninit::<[u8; 4]>::uninit();
        let slice = unsafe { &mut *buf.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(T::as_u32(unsafe { &buf.assume_init() }))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, PicoriError> { self.read_eu32::<NativeEndian>() }
    #[inline]
    fn read_bu32(&mut self) -> Result<u32, PicoriError> { self.read_eu32::<BigEndian>() }
    #[inline]
    fn read_lu32(&mut self) -> Result<u32, PicoriError> { self.read_eu32::<LittleEndian>() }
}

pub trait ReadArrayExtensionU8: Read {
    fn read_eu8_array<T: EndianAgnostic, const S: usize>(
        &mut self,
    ) -> Result<[u8; S], PicoriError> {
        let mut storage = MaybeUninit::<[u8; S]>::uninit();
        let slice = unsafe { &mut *storage.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(unsafe { storage.assume_init() })
    }

    #[inline]
    fn read_u8_array<const S: usize>(&mut self) -> Result<[u8; S], PicoriError> {
        self.read_eu8_array::<NativeEndian, S>()
    }
    #[inline]
    fn read_bu8_array<const S: usize>(&mut self) -> Result<[u8; S], PicoriError> {
        self.read_eu8_array::<BigEndian, S>()
    }
    #[inline]
    fn read_lu8_array<const S: usize>(&mut self) -> Result<[u8; S], PicoriError> {
        self.read_eu8_array::<LittleEndian, S>()
    }
}

pub trait ReadArrayExtensionU32: Read {
    fn read_eu32_array<T: EndianAgnostic, const S: usize>(
        &mut self,
    ) -> Result<[u32; S], PicoriError> {
        let mut storage = MaybeUninit::<[u32; S]>::uninit();
        let reference = unsafe { &mut *storage.as_mut_ptr() };
        let buf = unsafe { std::slice::from_raw_parts_mut(reference.as_mut_ptr() as *mut u8, 4 * S) };
        self.read_exact(buf)?;

        let storage = unsafe { &mut storage.assume_init() };
        for i in 0..S {
            storage[i] = T::as_u32(&buf[4 * i..4 * (i + 1)]);
        }

        Ok(*storage)
    }

    #[inline]
    fn read_u32_array<const S: usize>(&mut self) -> Result<[u32; S], PicoriError> {
        self.read_eu32_array::<NativeEndian, S>()
    }
    #[inline]
    fn read_bu32_array<const S: usize>(&mut self) -> Result<[u32; S], PicoriError> {
        self.read_eu32_array::<BigEndian, S>()
    }
    #[inline]
    fn read_lu32_array<const S: usize>(&mut self) -> Result<[u32; S], PicoriError> {
        self.read_eu32_array::<LittleEndian, S>()
    }
}

pub trait ReadStringExtension: Read {
    fn read_string<const L: usize, T: StringEncoding>(
        &mut self,
    ) -> Result<String, PicoriError> {
        let mut buf = unsafe { MaybeUninit::<[u8; L]>::uninit().assume_init() };
        self.read_exact(&mut buf)?;
        let str = T::decode_bytes(&buf)?;
        Ok(str)
    }
}

pub trait ReadExtension:
    ReadExtensionU8
    + ReadExtensionU32
    + ReadArrayExtensionU8
    + ReadArrayExtensionU32
    + ReadStringExtension
{
}

impl<T: Read + ?Sized> ReadExtensionU8 for T {}
impl<T: Read + ?Sized> ReadExtensionU32 for T {}
impl<T: Read + ?Sized> ReadArrayExtensionU8 for T {}
impl<T: Read + ?Sized> ReadArrayExtensionU32 for T {}
impl<T: Read + ?Sized> ReadStringExtension for T {}
impl<T: Read + ?Sized> ReadExtension for T {}
