//! GameCube [CISO][`crate::ciso`] (Compact ISO).
//!
//! [CISO][`crate::ciso`] (Compact ISO) is a simple format for reducing files
//! that contains large runs of zero bytes. The unerlying file is chunked into
//! `N` blocks (`N` <= 32760) for an arbitrary block size. Blocks with zero data
//! will be omitted from the compressed file.
//!
//! # Parse
//!
//! Because decompressed [CISO][`crate::ciso`] files can be rather large,
//! [`CisoReader`] is used to parse the file incrementally. The [`CisoReader`] will parse
//! the file header to determine how many blocks that are used. What you do with
//! the blocks is up to you, they can be access via [`CisoReader::blocks`]. To
//! decompress the whole file at once, use [`CisoReader::decompress`].
//!
//! ## Example
//!
//! This is an example of parse and decompress a [CISO][`crate::ciso`] file.
//!
//! ```no_run
//! # use std::fs::File;
//! # use std::fs::OpenOptions;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut input = File::open("compact_disc.iso")?;
//!     let mut reader = picori::CisoReader::new(&mut input)?;
//!
//!     let mut output = OpenOptions::new()
//!         .write(true)
//!         .create(true)
//!         .open("disc.iso")?;
//!     reader.decompress(&mut output)?;
//!     Ok(())
//! }
//! ```

use std::io::Write;
use std::panic::Location;

use crate::helper::{ParseProblem, Parser, ProblemLocation, Seeker};
use crate::Result;

/// [CISO][`crate::ciso`] magic number representing the four characters "CISO".
static MAGIC: u32 = 0x4F534943;

#[derive(Debug)]
struct Header {
    block_size: usize,
    blocks:     Vec<(u64, bool)>,
}

impl Header {
    pub fn from_binary<D: Parser + Seeker>(input: &mut D) -> Result<Self> {
        let magic = input.bu32()?;
        let block_size = input.bu32()? as usize;
        if magic != MAGIC {
            return Err(
                ParseProblem::InvalidMagic("expected: 0x4F534943", Location::current()).into(),
            );
        } else if block_size == 0 || block_size > 0x8000000 {
            return Err(ParseProblem::InvalidRange(
                "0 < block size <= 0x8000000",
                Location::current(),
            )
            .into());
        }

        let block_map = input.u8_array::<{ 0x8000 - 8 }>()?;
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
            Err(ParseProblem::InvalidHeader("invalid block map", Location::current()).into())
        }
    }
}

/// Reader for [CISO][`crate::ciso`] files.
pub struct CisoReader<'reader, D: Parser + Seeker> {
    header:      Header,
    reader:      &'reader mut D,
    data_offset: u64,
}

impl<'reader, D: Parser + Seeker> CisoReader<'reader, D> {
    /// Create a new [CISO][`crate::ciso`] reader from a binary stream.
    pub fn new(reader: &'reader mut D) -> Result<Self> {
        let header = Header::from_binary(reader)?;
        let data_offset = reader.position()?;
        Ok(Self {
            header,
            reader,
            data_offset,
        })
    }

    /// Get the block size of the [CISO][`crate::ciso`] file.
    pub fn block_size(&self) -> usize { self.header.block_size }

    /// Get the total size of the decompressed file.
    pub fn total_size(&self) -> usize { self.header.blocks.len() * self.header.block_size }

    /// Read data of block at index `index`. If the block is omitted, a zeroed
    /// buffer will be returned.
    pub fn read_block(&mut self, index: usize) -> Result<Vec<u8>> {
        let (offset, has_data) = self.header.blocks[index];
        let mut buffer = vec![0; self.header.block_size];
        if has_data {
            self.reader.goto(self.data_offset + offset)?;
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

    /// Decompress all [CISO][`crate::ciso`] block and write the data to a
    /// [`std::io::Write`]. If you need to know the final size of the
    /// decompressed file, use [`CisoReader::total_size`].
    pub fn decompress<Writer: Write>(&'reader mut self, writer: &mut Writer) -> Result<()> {
        self.blocks().try_for_each(|x| match x {
            Ok(x) => Ok(writer.write_all(&x)?),
            Err(e) => Err(e),
        })
    }
}

/// Iterator over all blocks of a [CISO][`crate::ciso`] file.
pub struct BlockIterator<'reader, 'x, D: Parser + Seeker> {
    reader: &'reader mut CisoReader<'x, D>,
    index:  usize,
}

impl<'reader, 'x, D: Parser + Seeker> Iterator for BlockIterator<'reader, 'x, D> {
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
