//! GameCube CISO (Compact ISO).
//!
//! CISO is also known as WIB. It is a compressed format that support any
//! underlying file format, i.e., must not be used to ISO files.
#![feature(negative_impls)]

use std::borrow::Cow;
use std::error::Error;
use std::f32::consts::E;
use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{bail, ensure, Result};
use flate2::{Decompress, FlushDecompress};
use itertools::Itertools;

use crate::helper::{read_lu32, read_lu64};
use crate::stream::{DeserializeError, DeserializeStream, Deserializeble};

/// CISO magic number representing the four characters "CISO".
static MAGIC: u32 = 0x4F534943;

#[derive(Debug)]
struct Header {
    magic:       u32,
    block_size:  usize,
    block_total: usize,
    blocks:      Vec<(usize, bool)>,
}

impl Deserializeble for Header {
    fn deserialize_stream<D: DeserializeStream>(input: &mut D) -> Result<Self, DeserializeError> {
        let mut magic = [0u8; 4];
        input.read_stream(&mut magic)?;
        let magic = u32::from_le_bytes(magic);

        let mut block_size = [0u8; 4];
        input.read_stream(&mut block_size)?;
        let block_size = u32::from_le_bytes(block_size) as usize;

        let block_map = [0u8; 0x8000 - 8];

        if magic != MAGIC {
            return Err(DeserializeError::InvalidHeader("invalid magic"));
        } else if block_size >= 0x8000 && block_size <= 0x80000000 {
            return Err(DeserializeError::InvalidHeader("invalid block size"));
        }

        let block_total = block_map
            .iter()
            .enumerate()
            .fold::<Option<usize>, _>(None, |acc, x| if x.1 == &1 { Some(x.0) } else { acc });
        let block_total = block_total.unwrap_or(0);

        let blocks = block_map
            .iter()
            .take(block_total)
            .enumerate()
            .map(|x| (x.0, *x.1 == 1))
            .collect();

        Ok(Header {
            magic,
            block_size,
            block_total,
            blocks,
        })
    }
}

pub struct CisoDecoder<'x, R: Read + Seek> {
    header:      Header,
    reader:      &'x mut R,
    data_offset: u64,
}

impl<'x, R: Read + Seek> CisoDecoder<'x, R> {
    pub fn new(reader: &'x mut R) -> Result<Self, DeserializeError> {
        let header = Header::deserialize_stream(reader)?;
        let data_offset = reader.stream_position()?;
        Ok(Self {
            header,
            reader,
            data_offset,
        })
    }

    pub fn decompress<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        self.reader.seek(SeekFrom::Start(self.data_offset))?;

        let zero_block = vec![0 as u8; self.header.block_size];
        let mut data_block = vec![0 as u8; self.header.block_size];
        self.header
            .blocks
            .iter()
            .try_for_each(|(_i, data_or_zero)| {
                println!(
                    "block: {:8} ({:8}) {}",
                    _i, self.header.block_total, data_or_zero
                );
                match data_or_zero {
                    true => {
                        self.reader.read_exact(&mut data_block)?;
                        writer.write_all(&data_block)?;
                    },
                    false => {
                        writer.write_all(&zero_block)?;
                    },
                }

                Ok(())
            })
    }
}
