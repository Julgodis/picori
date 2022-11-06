//! GameCube CISO (Compact ISO).
//!
//! CISO is also known as WIB. It is a compressed format that can to wrap any
//! other format, i.e., it must not be used we GameCube/Wii ISOs or
//! [GCM][`super::gcm`].
//!
//! ## Examples
//!
//! TODO: Add examples

use std::io::{Read, Seek, SeekFrom, Write};
use std::result::Result;

use crate::helper::error::{FormatError, PicoriError};
use crate::helper::{Deserializer, Seeker};

/// CISO magic number representing the four characters "CISO".
static MAGIC: u32 = 0x4F534943;

#[derive(Debug)]
struct Header {
    block_size: usize,
    blocks:     Vec<(usize, bool)>,
}

impl Header {
    pub fn deserialize<D: Deserializer + Seeker>(input: &mut D) -> Result<Self, PicoriError> {
        let magic = input.deserialize_bu32()?;
        let block_size = input.deserialize_bu32()? as usize;
        if magic != MAGIC {
            return Err(FormatError::InvalidHeader("invalid magic").into());
        } else if block_size == 0 || block_size > 0x800_0000 {
            return Err(FormatError::InvalidHeader("invalid block size").into());
        }

        let block_map = input.deserialize_u8_array::<{ 0x8000 - 8 }>()?;
        let block_total = block_map
            .iter()
            .enumerate()
            .filter_map(|(i, &x)| if x != 0 { Some(i + 1) } else { None })
            .max();

        if let Some(block_total) = block_total {
            let blocks = block_map
                .iter()
                .take(block_total)
                .enumerate()
                .map(|x| (x.0, *x.1 == 1))
                .collect();

            Ok(Header { block_size, blocks })
        } else {
            Err(FormatError::InvalidHeader("invalid block map").into())
        }
    }
}

pub struct CisoDecoder<'x, D: Deserializer + Seeker> {
    header:      Header,
    reader:      &'x mut D,
    data_offset: u64,
}

impl<'x, D: Deserializer + Seeker> CisoDecoder<'x, D> {
    pub fn new(reader: &'x mut D) -> Result<Self, PicoriError> {
        let header = Header::deserialize(reader)?;
        let data_offset = reader.position()?;
        Ok(Self {
            header,
            reader,
            data_offset,
        })
    }

    pub fn block_size(&self) -> usize { self.header.block_size }

    pub fn total_size(&self) -> usize { self.header.blocks.len() * self.header.block_size }

    pub fn decode_blocks<F, E>(&mut self, mut func: F) -> Result<(), E>
    where
        F: FnMut(usize, &[u8]) -> Result<(), E>,
        E: From<std::io::Error>,
    {
        match self.reader.seek(SeekFrom::Start(self.data_offset)) {
            Ok(_) => (),
            Err(PicoriError::IoError(e)) => return Err(e.into()),
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e).into()),
        }

        let zero_block = vec![0_u8; self.header.block_size];
        let mut data_block = vec![0_u8; self.header.block_size];
        self.header
            .blocks
            .iter()
            .enumerate()
            .map(|(i, x)| (i * self.header.block_size, x))
            .try_for_each(|(offset, (_, data_or_zero))| {
                match *data_or_zero {
                    true => {
                        match self.reader.read_into_buffer(&mut data_block) {
                            Ok(_) => (),
                            Err(PicoriError::IoError(e)) => return Err(e.into()),
                            Err(e) => {
                                return Err(std::io::Error::new(std::io::ErrorKind::Other, e).into())
                            },
                        }
                        func(offset, &data_block)?;
                    },
                    false => {
                        func(offset, &zero_block)?;
                    },
                }

                Ok(())
            })
    }

    pub fn decode<Writer>(&mut self, writer: &mut Writer) -> Result<(), PicoriError>
    where
        Writer: Write,
    {
        Ok(self.decode_blocks(|_, data| writer.write_all(data))?)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn ciso_decoder() {
        // example ciso
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);

        ciso[8 + 1] = 1;
        ciso.extend_from_slice(&[1, 1, 1, 1]);

        ciso[8 + 9] = 1;
        ciso.extend_from_slice(&[9, 9, 9, 9]);

        let mut reader = Cursor::new(ciso);
        let mut decoder = CisoDecoder::new(&mut reader).unwrap();
        assert!(decoder.block_size() == 0x04);
        assert!(decoder.total_size() == 0x04 * 10);

        let mut output = Vec::new();
        decoder.decode(&mut output).unwrap();
        assert_eq!(output.len(), 0x04 * 10);
        assert_eq!(output[0..4], [0, 0, 0, 0]);
        assert_eq!(output[4..8], [1, 1, 1, 1]);
        assert_eq!(output[8..12], [0, 0, 0, 0]);
        assert_eq!(output[12..16], [0, 0, 0, 0]);
        assert_eq!(output[16..20], [0, 0, 0, 0]);
        assert_eq!(output[20..24], [0, 0, 0, 0]);
        assert_eq!(output[24..28], [0, 0, 0, 0]);
        assert_eq!(output[28..32], [0, 0, 0, 0]);
        assert_eq!(output[32..36], [0, 0, 0, 0]);
        assert_eq!(output[36..40], [9, 9, 9, 9]);
    }

    #[test]
    fn invalid_magic() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0, 0, 0, 0]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(CisoDecoder::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn invalid_block_size() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0, 0, 0, 0]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(CisoDecoder::new(&mut Cursor::new(ciso)).is_err());

        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(CisoDecoder::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn invalid_blocks() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);
        assert!(CisoDecoder::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn decode_return_error() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);

        ciso[8 + 1] = 1;
        ciso.extend_from_slice(&[1, 1, 1, 1]);

        let mut reader = Cursor::new(ciso);
        let mut decoder = CisoDecoder::new(&mut reader).unwrap();
        let result =
            decoder.decode_blocks(|_, _| Err(std::io::Error::from(std::io::ErrorKind::Other)));
        assert!(result.is_err());

        let result = decoder.decode(&mut Cursor::new([0; 4]));
        assert!(result.is_err());
    }
}
