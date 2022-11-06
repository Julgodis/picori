//! Yaz0 compression and decompression.
//!
//! ## Compression
//!
//! TODO: Add documentation.
//!
//! ## Decompression
//!
//! TODO: Add documentation.
//!
//! ## References
//!
//! Implementation of the decompression algorithm is based
//! on compression specification <http://www.amnoid.de/gc/yaz0.txt> by amnoid.

use std::io::{Seek, SeekFrom};

use crate::error::{ensure, CompressionError, PicoriError};
use crate::helper::read_extension::ReadExtension;

pub struct Header {
    pub magic: u32,
    pub decompressed_size: u32,
    pub reserved0: u32,
    pub reserved1: u32,
}

impl Header {
    pub fn from_bytes<Reader>(input: &mut Reader) -> Result<Header, PicoriError>
    where
        Reader: ReadExtension,
    {
        Ok(Header {
            magic: input.read_bu32()?,
            decompressed_size: input.read_bu32()?,
            reserved0: input.read_bu32()?,
            reserved1: input.read_bu32()?,
        })
    }

    pub fn is_valid(&self) -> bool { self.magic == 0x59617A30 }
}

pub fn is_compressed<Reader>(input: &mut Reader) -> bool
where
    Reader: ReadExtension + Seek,
{
    let mut check = || -> Result<bool, PicoriError> {
        let base = input.stream_position()?;
        let is_compressed = Header::from_bytes(input).map(|header| header.is_valid());
        input.seek(SeekFrom::Start(base))?;
        is_compressed
    };

    check().unwrap_or(false)
}

pub fn decompress<Reader>(input: &mut Reader) -> Result<Vec<u8>, PicoriError>
where
    Reader: ReadExtension + Seek,
{
    let header = Header::from_bytes(input)?;
    ensure!(header.is_valid(), CompressionError::InvalidHeader());

    let current = input.stream_position()?;
    input.seek(SeekFrom::End(0))?;
    let compressed_size = input.stream_position()?;
    input.seek(SeekFrom::Start(current))?;

    // TODO: Use uninitialized memory
    let mut dest = vec![0_u8; header.decompressed_size as usize];
    let mut source = vec![0_u8; compressed_size as usize];
    input.read_exact(source.as_mut_slice())?;
    decompress_to_buffer(dest.as_mut_slice(), source.as_slice())?;

    Ok(dest)
}

pub fn decompress_to_buffer(dest: &mut [u8], source: &[u8]) -> Result<(), PicoriError> {
    let mut i = 0;
    let mut j = 0;

    loop {
        ensure!(i < source.len(), CompressionError::InvalidData());
        let code = source[i];
        i += 1;

        for k in 0..8 {
            if j >= dest.len() {
                return Ok(());
            }

            if (code & (0x80 >> k)) != 0 {
                ensure!(i < source.len(), CompressionError::InvalidData());
                dest[j] = source[i];
                i += 1;
                j += 1;
            } else {
                ensure!(i + 1 < source.len(), CompressionError::InvalidData());
                let byte0 = source[i] as usize;
                let byte1 = source[i + 1] as usize;
                i += 2;

                let a = (byte0 >> 4) & 0x0F;
                let b = byte0 & 0x0F;

                let mut length = a;
                if length == 0 {
                    ensure!(i < source.len(), CompressionError::InvalidData());
                    length = source[i] as usize + 0x12;
                    i += 1;
                } else {
                    length += 2;
                }

                let offset = ((b << 8) | byte1) + 1;
                ensure!(offset >= j, CompressionError::InvalidData());
                ensure!(
                    j - offset + length < dest.len(),
                    CompressionError::InvalidData()
                );
                for _ in 0..length {
                    dest[j] = dest[j - offset];
                    j += 1;
                }
            }
        }
    }
}
