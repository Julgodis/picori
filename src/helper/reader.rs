use std::io::Read;
use std::mem::MaybeUninit;

use crate::Result;

/// A helper trait for types that can read data into a buffer.
pub trait Reader {
    /// Read data into new [`Vec<u8>`].
    fn read_buffer(&mut self, size: usize) -> Result<Vec<u8>>;

    /// Read data into a mutable buffer.
    fn read_into_buffer(&mut self, buffer: &mut [u8]) -> Result<()>;

    /// Read data into a new fixed size buffer.
    fn read_fixed_buffer<const L: usize>(&mut self) -> Result<[u8; L]>;

    /// Read `L` items of type `T` from this reader or `L` * `sizeof(T)` bytes.
    ///
    /// In the future, this will be implemented using `read_fixed_buffer`
    /// instead of require the trait implementor to implement it. This is
    /// due to the requirement of feature `generic_const_expr` which is not
    /// yet stable.
    fn read_fixed_buffer_cge<T: Sized, const L: usize>(&mut self) -> Result<[T; L]>;
}

/// Implementation of [`Reader`] for all [`std::io::Read`].
impl<Base> Reader for Base
where
    Base: Read + Sized,
{
    fn read_buffer(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut data = unsafe {
            let mut data = Vec::with_capacity(size);
            data.set_len(size);
            data
        };

        self.read_exact(&mut data)?;
        Ok(data)
    }

    #[inline]
    fn read_into_buffer(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.read_exact(buffer)?;
        Ok(())
    }

    #[inline]
    fn read_fixed_buffer<const L: usize>(&mut self) -> Result<[u8; L]> {
        let mut buf = [0u8; L];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_fixed_buffer_cge<T: Sized, const L: usize>(&mut self) -> Result<[T; L]> {
        let byte_length = L * core::mem::size_of::<T>();
        let mut storage = MaybeUninit::<[T; L]>::uninit();
        let reference = unsafe { &mut *storage.as_mut_ptr() };
        let buf = unsafe {
            std::slice::from_raw_parts_mut(reference.as_mut_ptr() as *mut u8, byte_length)
        };
        self.read_exact(buf)?;
        Ok(unsafe { storage.assume_init() })
    }
}
