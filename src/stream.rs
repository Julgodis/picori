use std::io::{Read, Seek};
use std::result::Result;

use crate::string::StringError;

#[derive(thiserror::Error, Debug)]
pub enum DeserializeError {
    #[error("invalid header: {0}")]
    InvalidHeader(&'static str),

    #[error("invalid data: {0}")]
    InvalidData(&'static str),

    #[error("std::io::Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("StringError: {0}")]
    StringError(#[from] StringError),
}

pub trait DeserializeStream: Sized {
    fn read_stream(&mut self, buffer: &mut [u8]) -> Result<(), DeserializeError>;
    fn read_to_end_stream(&mut self, buffer: &mut Vec<u8>) -> Result<(), DeserializeError>;
}

impl<T: Read> DeserializeStream for T {
    #[inline]
    fn read_stream(&mut self, buffer: &mut [u8]) -> Result<(), DeserializeError> {
        self.read_exact(buffer)?;
        Ok(())
    }

    #[inline]
    fn read_to_end_stream(&mut self, buffer: &mut Vec<u8>) -> Result<(), DeserializeError> {
        self.read_to_end(buffer)?;
        Ok(())
    }
}

pub trait Deserializeble: Sized {
    fn deserialize_stream<D: Read + Seek>(input: &mut D) -> Result<Self, DeserializeError>;
}
