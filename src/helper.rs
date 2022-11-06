use std::io::{Read, SeekFrom, Seek};
use std::mem::MaybeUninit;

use crate::endian::{BigEndian, EndianAgnostic, LittleEndian, NativeEndian};
use crate::error::PicoriError;
use crate::string::{StringEncoding, StringError};
use crate::DeserializeError;

macro_rules! read_bu32 {
    ($buf:ident, $offset:expr) => {
        u32::from_be_bytes(
            $buf[$offset..($offset + 4)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_lu32 {
    ($buf:ident, $offset:expr) => {
        u32::from_le_bytes(
            $buf[$offset..($offset + 4)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_lu64 {
    ($buf:ident, $offset:expr) => {
        u64::from_le_bytes(
            $buf[$offset..($offset + 8)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_bu32_array {
    ($buf:ident, $offset:expr, $len:expr) => {
        $buf[$offset..($offset + 4 * $len)]
            .chunks_exact(4)
            .map(|chunk| u32::from_be_bytes(chunk.try_into().expect("slice with incorrect length")))
            .collect::<Vec<u32>>()
            .try_into()
            .expect("slice with incorrect length")
    };
}

pub(crate) use {read_bu32, read_bu32_array};

pub fn checked_add(a: u32, b: u32) -> Result<u32, PicoriError> {
    a.checked_add(b).ok_or(PicoriError::IntegerOverflow())
}

pub trait TakeLastN<T> {
    fn take_last_n(&self, n: usize) -> &[T];
}

impl<T> TakeLastN<T> for &[T] {
    fn take_last_n(&self, n: usize) -> &[T] {
        if self.len() < n {
            &self[..]
        } else {
            &self[self.len() - n..]
        }
    }
}

pub fn align_next(n: u32, alignment: u32) -> u32 {
    assert!(
        alignment.is_power_of_two(),
        "Alignment must be a power of two"
    );
    (n + alignment - 1) & !(alignment - 1)
}

pub trait ReadExtensionU8: Read {
    fn read_eu8<T: EndianAgnostic>(&mut self) -> Result<u8, DeserializeError> {
        let mut buf = MaybeUninit::<[u8; 1]>::uninit();
        let slice = unsafe { &mut *buf.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(T::as_u8(unsafe { &buf.assume_init() }))
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, DeserializeError> { self.read_eu8::<NativeEndian>() }
    #[inline]
    fn read_bu8(&mut self) -> Result<u8, DeserializeError> { self.read_eu8::<BigEndian>() }
    #[inline]
    fn read_lu8(&mut self) -> Result<u8, DeserializeError> { self.read_eu8::<LittleEndian>() }
}

pub trait ReadExtensionU32: Read {
    fn read_eu32<T: EndianAgnostic>(&mut self) -> Result<u32, DeserializeError> {
        let mut buf = MaybeUninit::<[u8; 4]>::uninit();
        let slice = unsafe { &mut *buf.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(T::as_u32(unsafe { &buf.assume_init() }))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, DeserializeError> { self.read_eu32::<NativeEndian>() }
    #[inline]
    fn read_bu32(&mut self) -> Result<u32, DeserializeError> { self.read_eu32::<BigEndian>() }
    #[inline]
    fn read_lu32(&mut self) -> Result<u32, DeserializeError> { self.read_eu32::<LittleEndian>() }
}

pub trait ReadArrayExtensionU8: Read {
    fn read_eu8_array<T: EndianAgnostic, const S: usize>(
        &mut self,
    ) -> Result<[u8; S], DeserializeError> {
        let mut storage = MaybeUninit::<[u8; S]>::uninit();
        let slice = unsafe { &mut *storage.as_mut_ptr() };
        self.read_exact(slice)?;
        Ok(unsafe { storage.assume_init() })
    }

    #[inline]
    fn read_u8_array<const S: usize>(&mut self) -> Result<[u8; S], DeserializeError> {
        self.read_eu8_array::<NativeEndian, S>()
    }
    #[inline]
    fn read_bu8_array<const S: usize>(&mut self) -> Result<[u8; S], DeserializeError> {
        self.read_eu8_array::<BigEndian, S>()
    }
    #[inline]
    fn read_lu8_array<const S: usize>(&mut self) -> Result<[u8; S], DeserializeError> {
        self.read_eu8_array::<LittleEndian, S>()
    }
}

pub trait ReadArrayExtensionU32: Read {
    fn read_eu32_array<T: EndianAgnostic, const S: usize>(
        &mut self,
    ) -> Result<[u32; S], DeserializeError> {
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
    fn read_u32_array<const S: usize>(&mut self) -> Result<[u32; S], DeserializeError> {
        self.read_eu32_array::<NativeEndian, S>()
    }
    #[inline]
    fn read_bu32_array<const S: usize>(&mut self) -> Result<[u32; S], DeserializeError> {
        self.read_eu32_array::<BigEndian, S>()
    }
    #[inline]
    fn read_lu32_array<const S: usize>(&mut self) -> Result<[u32; S], DeserializeError> {
        self.read_eu32_array::<LittleEndian, S>()
    }
}

pub trait ReadStringExtension: Read {
    fn read_string<const L: usize, T: StringEncoding>(
        &mut self,
    ) -> Result<String, DeserializeError> {
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

//
//
//

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

pub(crate) use ensure;

//
//
//

pub struct SliceReader<'a> {
    slice: &'a [u8],
    index: u64,
    length: u64,
}

impl<'a> SliceReader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, index: 0, length: slice.len() as u64 }
    }
}

impl<'a> Read for SliceReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let wanted = buf.len() as u64;
        let remaining = self.length - self.index;
        ensure!(remaining >= wanted, std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));

        let start = self.index as usize;
        let end = (self.index + wanted) as usize;
        buf.copy_from_slice(&self.slice[start..end]);
        self.index += wanted;
        Ok(wanted as usize)
    }
}

impl<'a> Seek for SliceReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        let new_index = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => (self.length as i64) + pos,
            SeekFrom::Current(pos) => (self.index as i64) + pos,
        };

        ensure!(new_index >= 0, std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid seek position"));

        let new_index = new_index as u64;
        ensure!(new_index <= self.length, std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));

        self.index = new_index;
        Ok(self.index)
    }
}