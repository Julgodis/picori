//! Yaz0 compression and decompression.
//!
//! ## Compression
//!
//! Compression is not yet supported.
//!
//! ## Decompression
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("data.yaz0")?;
//!     let reader = picori::Yaz0Reader::new(file)?;
//!     let _ = reader.decompress()?;
//!     Ok(())
//! }
//! ```
//! 
//! ## References
//!
//! [Yaz0](http://www.amnoid.de/gc/yaz0.txt) - Implementation of the decompression algorithm is based
//! on specification and format description by Amnoid.

use std::io::SeekFrom;

use crate::error::DecompressionProblem::*;
use crate::helper::{ensure, Parser, Seeker};
use crate::Result;

struct Header {
    magic: u32,
    decompressed_size: u32,
    _reserved0: u32,
    _reserved1: u32,
}

impl Header {
    fn from_binary<D: Parser>(input: &mut D) -> Result<Header> {
        Ok(Header {
            magic: input.deserialize_bu32()?,
            decompressed_size: input.deserialize_bu32()?,
            _reserved0: input.deserialize_bu32()?,
            _reserved1: input.deserialize_bu32()?,
        })
    }

    fn is_valid(&self) -> bool { self.magic == 0x59617A30 }
}

/// Decompresses a Yaz0 compressed file.
pub struct Yaz0Reader<D: Parser + Seeker> {
    reader: D,
    decompressed_size: u32,
}

impl<D: Parser + Seeker> Yaz0Reader<D> {
    /// Creates a new Yaz0 reader.
    pub fn new(mut reader: D) -> Result<Yaz0Reader<D>> {
        let header = Header::from_binary(&mut reader)?;
        ensure!(header.is_valid(), InvalidHeader("invalid magic"));
        Ok(Yaz0Reader {
            reader,
            decompressed_size: header.decompressed_size,
        })
    }

    /// Decompressed size of the data.
    pub fn decompressed_size(&self) -> u32 { self.decompressed_size }

    /// Decompresses the data into a new allocated [`Vec`].
    pub fn decompress(&mut self) -> Result<Vec<u8>> {
        let mut output = vec![0; self.decompressed_size as usize];
        self.decompress_into(output.as_mut_slice())?;
        Ok(output)
    }

    /// Decompresses the data into the given buffer. The buffer must be large
    /// enough to hold the decompressed data. Use
    /// [`Yaz0Reader::decompressed_size`] to get the size of the decompressed.
    pub fn decompress_into(&mut self, destination: &mut [u8]) -> Result<()> {
        ensure!(
            destination.len() as u32 >= self.decompressed_size,
            InvalidDecompressedSize
        );

        let size = self.decompressed_size as usize;
        let mut dest = 0;
        let mut code = 0;
        let mut code_bits = 0;

        while dest < size {
            if code_bits == 0 {
                code = self.reader.deserialize_u8()? as u32;
                code_bits = 8;
            }

            if code & 0x80 != 0 {
                let byte = self.reader.deserialize_u8()?;
                destination[dest] = byte;
                dest += 1;
            } else {
                let byte0 = self.reader.deserialize_u8()?;
                let byte1 = self.reader.deserialize_u8()?;
                let a = (byte0 & 0xf) as usize;
                let b = (byte0 >> 4) as usize;
                let offset = (a << 8) | (byte1 as usize);
                let length = match b {
                    0 => (self.reader.deserialize_u8()? as usize) + 0x12,
                    length => length + 2,
                };

                ensure!(offset < dest, UnexpectedEndOfData);
                let base = dest - (offset + 1);
                for n in 0..length {
                    destination[dest] = destination[base + n];
                    dest += 1;
                }
            }

            code <<= 1;
            code_bits -= 1;
        }

        Ok(())
    }
}

/// Check if the given data is compressed with Yaz0.
pub fn is_yaz0<D: Parser + Seeker>(input: &mut D) -> bool {
    let mut check = || -> Result<bool> {
        let base = input.position()?;
        let header = Header::from_binary(input)?;
        let is_compressed = header.is_valid();
        input.seek(SeekFrom::Start(base))?;
        Ok(is_compressed)
    };

    check().unwrap_or(false)
}
