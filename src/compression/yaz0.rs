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
//! [Yaz0](http://www.amnoid.de/gc/yaz0.txt) - Implementation of the decompression algorithm is based
//! on specification and format description by Amnoid.

use std::io::SeekFrom;

use crate::error::DecompressionProblem::*;
use crate::helper::{ensure, Deserializer, Seeker};
use crate::Result;

pub struct Header {
    magic: u32,
    decompressed_size: u32,
    _reserved0: u32,
    _reserved1: u32,
}

impl Header {
    pub fn from_bytes<D: Deserializer>(input: &mut D) -> Result<Header> {
        Ok(Header {
            magic: input.deserialize_bu32()?,
            decompressed_size: input.deserialize_bu32()?,
            _reserved0: input.deserialize_bu32()?,
            _reserved1: input.deserialize_bu32()?,
        })
    }

    pub fn is_valid(&self) -> bool { self.magic == 0x59617A30 }
}

pub fn is_compressed<D: Deserializer + Seeker>(input: &mut D) -> bool {
    let mut check = || -> Result<bool> {
        let base = input.position()?;
        let is_compressed = Header::from_bytes(input).map(|header| header.is_valid());
        input.seek(SeekFrom::Start(base))?;
        is_compressed
    };

    check().unwrap_or(false)
}

pub fn decompress<D: Deserializer + Seeker>(input: &mut D) -> Result<Vec<u8>> {
    let header = Header::from_bytes(input)?;
    ensure!(header.is_valid(), InvalidHeader("invalid yaz0 header"));

    let current = input.position()?;
    input.seek(SeekFrom::End(0))?;
    let compressed_size = input.position()?;
    input.seek(SeekFrom::Start(current))?;

    let mut dest = vec![0_u8; header.decompressed_size as usize];
    let mut source = vec![0_u8; compressed_size as usize];
    input.read_into_buffer(source.as_mut_slice())?; // TODO: use `read_buffer`
    decompress_to_buffer(dest.as_mut_slice(), source.as_slice())?;

    Ok(dest)
}

pub fn decompress_to_buffer(dest: &mut [u8], source: &[u8]) -> Result<()> {
    let mut i = 0;
    let mut j = 0;

    loop {
        ensure!(i < source.len(), UnexpectedEndOfData);
        let code = source[i];
        i += 1;

        for k in 0..8 {
            if j >= dest.len() {
                return Ok(());
            }

            if (code & (0x80 >> k)) != 0 {
                ensure!(i < source.len(), UnexpectedEndOfData);
                dest[j] = source[i];
                i += 1;
                j += 1;
            } else {
                ensure!(i + 1 < source.len(), UnexpectedEndOfData);
                let byte0 = source[i] as usize;
                let byte1 = source[i + 1] as usize;
                i += 2;

                let a = (byte0 >> 4) & 0x0F;
                let b = byte0 & 0x0F;

                let mut length = a;
                if length == 0 {
                    ensure!(i < source.len(), UnexpectedEndOfData);
                    length = source[i] as usize + 0x12;
                    i += 1;
                } else {
                    length += 2;
                }

                let offset = ((b << 8) | byte1) + 1;
                ensure!(offset >= j, UnexpectedEndOfData);
                ensure!(j - offset + length < dest.len(), UnexpectedEndOfData);
                for _ in 0..length {
                    dest[j] = dest[j - offset];
                    j += 1;
                }
            }
        }
    }
}
