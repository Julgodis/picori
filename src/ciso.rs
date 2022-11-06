//! GameCube CISO (Compact ISO).
//!
//! CISO (Compact ISO) is a simple format for reducing files that contains large
//! runs of zero bytes. The unerlying file is chunked into N blocks (N <= 32760)
//! for an arbitrary block size. Blocks with zero data will be omitted from the
//! compressed file.
//!
//! ## Examples
//!
//! TODO: Add examples

use std::io::{SeekFrom, Write};

use crate::helper::{Deserializer, ParseProblem, ProblemLocation, Seeker};
use crate::Result;

/// CISO magic number representing the four characters "CISO".
static MAGIC: u32 = 0x4F534943;

#[derive(Debug)]
struct Header {
    block_size: usize,
    blocks:     Vec<(u64, bool)>,
}

impl Header {
    pub fn deserialize<D: Deserializer + Seeker>(input: &mut D) -> Result<Self> {
        let magic = input.deserialize_bu32()?;
        let block_size = input.deserialize_bu32()? as usize;
        if magic != MAGIC {
            return Err(ParseProblem::InvalidMagic(
                "expected: 0x4F534943",
                std::panic::Location::current(),
            )
            .into());
        } else if block_size == 0 || block_size > 0x8000000 {
            return Err(ParseProblem::InvalidRange(
                "0 < block size <= 0x8000000",
                std::panic::Location::current(),
            )
            .into());
        }

        let block_map = input.deserialize_u8_array::<{ 0x8000 - 8 }>()?;
        let block_total = block_map
            .iter()
            .enumerate()
            .filter_map(|(i, &x)| if x != 0 { Some(i + 1) } else { None })
            .max();

        if let Some(block_total) = block_total {
            let mut blocks = block_map
                .iter()
                .take(block_total)
                .enumerate()
                .map(|x| (0_u64, *x.1 == 1))
                .collect::<Vec<_>>();

            let mut offset = 0_u64;
            for block in blocks.iter_mut() {
                if block.1 {
                    block.0 = offset;
                    offset += block_size as u64;
                }
            }

            Ok(Header { block_size, blocks })
        } else {
            Err(
                ParseProblem::InvalidHeader("invalid block map", std::panic::Location::current())
                    .into(),
            )
        }
    }
}

/// Parse CISO files and provides a simple API to access each block.
///
/// # Examples
/// TODO: Add examples
pub struct Reader<'reader, D: Deserializer + Seeker> {
    header:      Header,
    reader:      &'reader mut D,
    data_offset: u64,
}

impl<'reader, D: Deserializer + Seeker> Reader<'reader, D> {
    /// Create a new CISO reader from a [`Deserializer`] + [`Seeker`].
    pub fn new(reader: &'reader mut D) -> Result<Self> {
        let header = Header::deserialize(reader)?;
        let data_offset = reader.position()?;
        Ok(Self {
            header,
            reader,
            data_offset,
        })
    }

    /// Get the block size of the CISO file.
    pub fn block_size(&self) -> usize { self.header.block_size }

    /// Get the total size of the decompressed file.
    pub fn total_size(&self) -> usize { self.header.blocks.len() * self.header.block_size }

    /// Read data of block at index `index`. If the block is omitted, a zeroed
    /// buffer will be returned.
    pub fn read_block(&mut self, index: usize) -> Result<Vec<u8>> {
        let (offset, has_data) = self.header.blocks[index];
        let mut buffer = vec![0; self.header.block_size];
        if has_data {
            self.reader
                .seek(SeekFrom::Start(self.data_offset + offset))?;
            self.reader.read_into(&mut buffer)?;
        }
        Ok(buffer)
    }

    /// Return an iterator over all blocks that returns their data.
    pub fn blocks<'this>(&'this mut self) -> BlockIterator<'this, 'reader, D> {
        BlockIterator {
            reader: self,
            index:  0,
        }
    }

    /// Decompress all CISO block and write the data to [`std::io::Write`]. If
    /// you need to know the final size of the decompressed file, use
    /// [`Reader::total_size`].
    pub fn decompress<Writer: Write>(&'reader mut self, writer: &mut Writer) -> Result<()> {
        self.blocks().try_for_each(|x| match x {
            Ok(x) => Ok(writer.write_all(&x)?),
            Err(e) => Err(e),
        })
    }
}

/// Iterator over all blocks of a CISO file.
pub struct BlockIterator<'reader, 'x, D: Deserializer + Seeker> {
    reader: &'reader mut Reader<'x, D>,
    index:  usize,
}

impl<'reader, 'x, D: Deserializer + Seeker> Iterator for BlockIterator<'reader, 'x, D> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.reader.header.blocks.len() {
            None
        } else {
            let data = self.reader.read_block(self.index);
            self.index += 1;
            Some(data)
        }
    }
}
