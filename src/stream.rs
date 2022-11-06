use std::io::Read;
use std::result::Result;

#[derive(thiserror::Error, Debug)]
pub enum DeserializeError {
    #[error("invalid header")]
    InvalidHeader(&'static str),

    #[error("io error")]
    IoError(#[source] std::io::Error),
}

impl From<std::io::Error> for DeserializeError {
    fn from(error: std::io::Error) -> Self { DeserializeError::IoError(error) }
}

pub trait DeserializeStream: Sized {
    fn read_stream(&mut self, buffer: &mut [u8]) -> Result<(), DeserializeError>;
}

impl<T: Read> DeserializeStream for T {
    #[inline]
    fn read_stream(&mut self, buffer: &mut [u8]) -> Result<(), DeserializeError> {
        self.read_exact(buffer)?;
        Ok(())
    }
}

pub trait Deserializeble: Sized {
    fn deserialize_stream<D: DeserializeStream>(input: &mut D) -> Result<Self, DeserializeError>;
}
