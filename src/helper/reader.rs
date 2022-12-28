use std::io::Read;
use std::panic::Location;

use crate::{Error, Result};

/// A helper trait for types that can read data into a buffer.
pub trait Reader: Read {
    /// Read data into a mutable buffer.
    #[track_caller]
    #[inline]
    fn read_into(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.read_into_tracked(buffer, Location::caller())
    }

    /// Read data into a mutable buffer. With caller location.
    #[inline]
    fn read_into_tracked(
        &mut self,
        buffer: &mut [u8],
        caller: &'static std::panic::Location,
    ) -> Result<()> {
        match self.read_exact(buffer) {
            Ok(..) => Ok(()),
            Err(io) => Err(Error::ReadFailed(buffer.len(), io, caller)),
        }
    }

    /// Read data into new buffer of u8.
    #[track_caller]
    #[inline]
    fn read_as_vec(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut vec = vec![0u8; size];
        self.read_into_tracked(vec.as_mut_slice(), Location::caller())?;
        Ok(vec)
    }

    /// Read `L` items of type `T` from this reader or `L` * `sizeof(T)` bytes.
    #[track_caller]
    #[inline]
    fn read_buffer_of<T: Sized, const L: usize>(&mut self) -> Result<[T; L]>
    where
        T: Copy + Default,
    {
        self.read_buffer_of_tracked::<T, L>(Location::caller())
    }

    /// Read `L` items of type `T` from this reader or `L` * `sizeof(T)` bytes.
    /// With caller location.
    #[inline]
    fn read_buffer_of_tracked<T: Sized, const L: usize>(
        &mut self,
        caller: &'static std::panic::Location,
    ) -> Result<[T; L]>
    where
        T: Copy + Default,
    {
        let length = L * core::mem::size_of::<T>();
        let mut buffer = [T::default(); L];
        let ptr = buffer.as_mut_ptr() as *mut u8;
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, length) };
        self.read_into_tracked(slice, caller)?;
        Ok(buffer)
    }
}

impl Reader for std::fs::File {}
impl<T: Reader> Reader for std::io::BufReader<T> {}
impl<T> Reader for std::io::Cursor<T>
where
    Self: Read,
    T: AsRef<[u8]>,
{
}
