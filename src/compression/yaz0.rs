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
//! on compression specification [http://www.amnoid.de/gc/yaz0.txt] by amnoid.

use anyhow::{ensure, Result};

use crate::error::DecompressionError;
use crate::helper::read_bu32;

pub struct Header {
    pub magic:           u32,
    pub decompress_size: u32,
    pub reserved0:       u32,
    pub reserved1:       u32,
}

impl Header {
    pub fn from_bytes(input: &[u8]) -> Result<Header> {
        ensure!(input.len() >= 16, DecompressionError::InvalidHeader());

        Ok(Header {
            magic:           read_bu32!(input, 0),
            decompress_size: read_bu32!(input, 4),
            reserved0:       read_bu32!(input, 8),
            reserved1:       read_bu32!(input, 12),
        })
    }

    pub fn is_valid(&self) -> bool { self.magic == 0x59617A30 }
}

pub fn is_compressed(input: &[u8]) -> bool {
    Header::from_bytes(input)
        .map(|header| header.is_valid())
        .unwrap_or(false)
}


pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    let header = Header::from_bytes(input)?;
    ensure!(header.is_valid(), DecompressionError::InvalidHeader());

    let mut output = vec![0 as u8; header.decompress_size as usize];

    ensure!(input.len() >= 16, DecompressionError::MissingData());
    decompress_to_buffer(&mut output[..], &input[16..])?;

    Ok(output)
}

pub fn decompress_to_buffer(dest: &mut [u8], source: &[u8]) -> Result<()> {
    let mut i = 0;
    let mut j = 0;

    loop {
        ensure!(i < source.len(), DecompressionError::InvalidSourceOffset());
        let code = source[i];
        i += 1;

        for k in 0..8 {
            if j >= dest.len() {
                return Ok(());
            }

            if (code & (0x80 >> k)) != 0 {
                ensure!(i < source.len(), DecompressionError::InvalidSourceOffset());
                dest[j] = source[i];
                i += 1;
                j += 1;
            } else {
                ensure!(
                    i + 1 < source.len(),
                    DecompressionError::InvalidSourceOffset()
                );
                let byte0 = source[i + 0] as usize;
                let byte1 = source[i + 1] as usize;
                i += 2;

                let a = (byte0 >> 4) & 0x0F;
                let b = (byte0 >> 0) & 0x0F;

                let mut length = a as usize;
                if length == 0 {
                    ensure!(i < source.len(), DecompressionError::InvalidSourceOffset());
                    length = source[i] as usize + 0x12;
                    i += 1;
                } else {
                    length += 2;
                }

                let offset = ((b << 8) | byte1) + 1;
                ensure!(offset >= j, DecompressionError::InvalidDestinationOffset());
                ensure!(
                    j - offset + length < dest.len(),
                    DecompressionError::InvalidDestinationOffset()
                );
                for _ in 0..length {
                    dest[j] = dest[j - offset];
                    j += 1;
                }
            }
        }
    }
}
