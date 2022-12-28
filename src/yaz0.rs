//! Yaz0 compression and decompression.
//!
//! ## Compression
//!
//! Compression is not yet supported.
//!
//! ## Decompression
//!
//! Decompress a Yaz0 compressed file:
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("data.yaz0")?;
//!     let mut reader = picori::Yaz0Reader::new(file)?;
//!     // use `reader` to read the decompressed data like any other file
//!     Ok(())
//! }
//! ```
//!
//! Alternatively, you can use the [`decompress`] or [`decompress_into`] function to decompress:
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("data.yaz0")?;
//!     let decompressed_size = picori::yaz0::Header::decompressed_size(&mut file)?;
//!     let decompressed = picori::yaz0::decompress(&mut file, decompressed_size)?;
//!     Ok(())
//! }
//! ```
//!
//! ## References
//!
//! [Yaz0](http://www.amnoid.de/gc/yaz0.txt) - Implementation of the Yaz0 decompression is based
//! on the specification and format description by Amnoid.

use std::io::{Read, Seek};
use std::panic::Location;

use crate::error::DecompressionProblem::*;
use crate::helper::{ensure, Parser, ProblemLocation, Seeker};
use crate::{Reader, Result};

/// Yaz0 header.
pub struct Header {
    /// Yaz0 magic (0x59617A30).
    pub magic: u32,
    /// Size of decompressed data.
    pub decompressed_size: u32,
    _reserved0: u32,
    _reserved1: u32,
}

impl Header {
    /// Reads a Yaz0 header from a reader.
    pub fn from_binary<D: Parser>(input: &mut D) -> Result<Header> {
        Ok(Header {
            magic: input.bu32()?,
            decompressed_size: input.bu32()?,
            _reserved0: input.bu32()?,
            _reserved1: input.bu32()?,
        })
    }

    /// Checks if the header is valid.
    pub fn is_valid(&self) -> bool {
        self.magic == 0x59617A30
    }

    pub fn decompressed_size(input: &mut impl Parser) -> Result<usize> {
        let header = Header::from_binary(input)?;
        ensure!(
            header.is_valid(),
            InvalidHeader("Invalid magic", Location::current())
        );
        Ok(header.decompressed_size as usize)
    }
}

/// Decompresses a Yaz0 compressed file.
pub struct Yaz0Reader<D: Parser + Seeker> {
    reader: D,
    decompressed: Vec<u8>,
    position: usize,
    transparent: bool,
}

impl<D: Parser + Seeker> Yaz0Reader<D> {
    /// Creates a new Yaz0 reader.
    pub fn new(mut reader: D) -> Result<Yaz0Reader<D>> {
        let base = reader.position()?;
        let header = Header::from_binary(&mut reader);
        if header.as_ref().map(|x| x.is_valid()).unwrap_or(false) {
            let header = header.unwrap();
            let data = decompress(&mut reader, header.decompressed_size as usize)?;
            Ok(Yaz0Reader {
                reader,
                decompressed: data,
                position: 0,
                transparent: false,
            })
        } else {
            reader.goto(base)?;
            Ok(Yaz0Reader {
                reader,
                decompressed: Vec::new(),
                position: 0,
                transparent: true,
            })
        }
    }

    /// Decompressed size of the data.
    pub fn decompressed_size(&self) -> usize {
        self.decompressed.len()
    }
}

impl<D: Parser + Seeker + Read> Read for Yaz0Reader<D> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.transparent {
            self.reader.read(buf)
        } else {
            let size = buf.len();
            let mut n = 0;
            while n < size && self.position < self.decompressed.len() {
                buf[n] = self.decompressed[self.position];
                n += 1;
                self.position += 1;
            }
            Ok(n)
        }
    }
}

impl<D: Parser + Seeker + Seek> Seek for Yaz0Reader<D> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        if self.transparent {
            self.reader.seek(pos)
        } else {
            let position = match pos {
                std::io::SeekFrom::Start(n) => Ok(n as usize),
                std::io::SeekFrom::Current(n) => {
                    let absolute = self.position as i64 + n;
                    if absolute < 0 {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "invalid seek to a negative or overflowing position",
                        ))
                    } else {
                        Ok(absolute as usize)
                    }
                },
                std::io::SeekFrom::End(n) => {
                    let absolute = self.decompressed.len() as i64 + n;
                    if absolute < 0 {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "invalid seek to a negative or overflowing position",
                        ))
                    } else {
                        Ok(absolute as usize)
                    }
                },
            }?;

            if position > self.decompressed.len() {
                self.position = self.decompressed.len();
            } else {
                self.position = position;
            }

            Ok(self.position as u64)
        }
    }
}

impl<D: Parser + Seeker> Reader for Yaz0Reader<D> {}
impl<D: Parser + Seeker> Seeker for Yaz0Reader<D> {}
impl<D: Parser + Seeker> Parser for Yaz0Reader<D> {}

/// Check if the given data is compressed with Yaz0 by looking for the Yaz0
/// magic.
pub fn is_yaz0<D: Parser + Seeker>(input: &mut D) -> bool {
    let mut check = || -> Result<bool> {
        let base = input.position()?;
        let header = Header::from_binary(input)?;
        let is_compressed = header.is_valid();
        input.goto(base)?;
        Ok(is_compressed)
    };

    check().unwrap_or(false)
}

/// Decompresses the data into a new allocated [`Vec`]. `decompressed_size` can be determined
/// by looking at the Yaz0 header [`Header`].
pub fn decompress<D: Parser + Seeker>(input: &mut D, decompressed_size: usize) -> Result<Vec<u8>> {
    let mut output = vec![0; decompressed_size];
    decompress_into(input, output.as_mut_slice())?;
    Ok(output)
}

/// Decompresses the data into the given buffer. The buffer must be large
/// enough to hold the decompressed data.
pub fn decompress_into<D: Parser + Seeker>(input: &mut D, destination: &mut [u8]) -> Result<()> {
    let decompressed_size = destination.len();
    let size = decompressed_size as usize;
    let mut dest = 0;
    let mut code = 0;
    let mut code_bits = 0;

    while dest < size {
        if code_bits == 0 {
            code = input.u8()? as u32;
            code_bits = 8;
        }

        if code & 0x80 != 0 {
            let byte = input.u8()?;
            destination[dest] = byte;
            dest += 1;
        } else {
            let byte0 = input.u8()?;
            let byte1 = input.u8()?;
            let a = (byte0 & 0xf) as usize;
            let b = (byte0 >> 4) as usize;
            let offset = (a << 8) | (byte1 as usize);
            let length = match b {
                0 => (input.u8()? as usize) + 0x12,
                length => length + 2,
            };

            ensure!(offset < dest, UnexpectedEndOfData(Location::current()));
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
