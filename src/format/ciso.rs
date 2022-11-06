//! GameCube CISO (Compact ISO).
//!
//! CISO is also known as WIB. It is a compressed format that can to wrap any
//! other format, i.e., it must not be used we GameCube/Wii ISOs or
//! [GCM][`super::gcm`].
//!
//! ## Examples
//!
//! TODO: Add examples

use std::io::{Read, Seek};
use std::result::Result;

use crate::error::{FormatError, PicoriError};
use crate::helper::read_extension::ReadExtension;

/// CISO magic number representing the four characters "CISO".
static MAGIC: u32 = 0x4F534943;

#[derive(Debug)]
struct Header {
    _block_size: usize,
    _blocks:     Vec<(usize, bool)>,
}

impl Header {
    pub fn deserialize<Reader>(input: &mut Reader) -> Result<Self, PicoriError>
    where
        Reader: ReadExtension + Seek,
    {
        let magic = input.read_bu32()?;
        let block_size = input.read_bu32()? as usize;
        if magic != MAGIC {
            return Err(FormatError::InvalidHeader("invalid magic").into());
        } else if block_size <= 0x8000 || block_size > 0x80000000 {
            return Err(FormatError::InvalidHeader("invalid block size").into());
        }

        let block_map = input.read_bu8_array::<{ 0x8000 - 8 }>()?;
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
            _block_size: block_size,
            _blocks:     blocks,
        })
    }
}

pub struct CisoDecoder<'x, R: Read + Seek> {
    _header:      Header,
    _reader:      &'x mut R,
    _data_offset: u64,
}

impl<'x, Reader> CisoDecoder<'x, Reader>
where
    Reader: ReadExtension + Seek,
{
    pub fn new(reader: &'x mut Reader) -> Result<Self, PicoriError> {
        let header = Header::deserialize(reader)?;
        let data_offset = reader.stream_position()?;
        Ok(Self {
            _header:      header,
            _reader:      reader,
            _data_offset: data_offset,
        })
    }

    // pub fn decode<Writer>(&mut self, writer: &mut Writer) -> Result<(),
    // PicoriError> where
    // Writer: Write,
    // {
    // self.reader.seek(SeekFrom::Start(self.data_offset))?;
    //
    // let zero_block = vec![0_u8; self.header.block_size];
    // let mut data_block = vec![0_u8; self.header.block_size];
    // self.header
    // .blocks
    // .iter()
    // .try_for_each(|(_i, data_or_zero)| {
    // match data_or_zero {
    // true => {
    // self.reader.read_exact(&mut data_block)?;
    // writer.write_all(&data_block)?;
    // },
    // false => {
    // writer.write_all(&zero_block)?;
    // },
    // }
    //
    // Ok(())
    // })
    // }
}
