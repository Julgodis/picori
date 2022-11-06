use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

use crate::error::ensure;

pub struct SliceReader<'a> {
    slice:  &'a [u8],
    index:  u64,
    length: u64,
}

impl<'a> SliceReader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            slice,
            index: 0,
            length: slice.len() as u64,
        }
    }
}

impl<'a> Read for SliceReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let wanted = buf.len() as u64;
        let remaining = self.length - self.index;
        ensure!(
            remaining >= wanted,
            Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF")
        );

        let start = self.index as usize;
        let end = (self.index + wanted) as usize;
        buf.copy_from_slice(&self.slice[start..end]);
        self.index += wanted;
        Ok(wanted as usize)
    }
}

impl<'a> Seek for SliceReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        let new_index = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => (self.length as i64) + pos,
            SeekFrom::Current(pos) => (self.index as i64) + pos,
        };

        ensure!(
            new_index >= 0,
            Error::new(ErrorKind::InvalidInput, "Invalid seek position")
        );

        let new_index = new_index as u64;
        ensure!(
            new_index <= self.length,
            Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF")
        );

        self.index = new_index;
        Ok(self.index)
    }
}
